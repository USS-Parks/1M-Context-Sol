Set-StrictMode -Version Latest

$script:BaselineTokens = 12000L

function ConvertFrom-ContextTokenEvent {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string[]] $Lines,

        [datetime] $Now = [datetime]::UtcNow,

        [int] $StaleAfterSeconds = 300,

        [Nullable[long]] $PreviousUsedTokens
    )

    $event = $null
    for ($index = $Lines.Count - 1; $index -ge 0; $index--) {
        try {
            $candidate = $Lines[$index] | ConvertFrom-Json -ErrorAction Stop
            if ($candidate.type -eq 'event_msg' -and $candidate.payload.type -eq 'token_count') {
                $event = $candidate
                break
            }
        }
        catch {
            continue
        }
    }

    if ($null -eq $event) {
        throw 'No valid token_count event was found.'
    }

    $usedTokens = [long]$event.payload.info.last_token_usage.total_tokens
    $contextWindow = [long]$event.payload.info.model_context_window
    if ($contextWindow -le $script:BaselineTokens -or $usedTokens -lt 0) {
        throw 'The token event contains an invalid context window or active-context count.'
    }

    $effectiveWindow = $contextWindow - $script:BaselineTokens
    $adjustedUsed = [math]::Max($usedTokens - $script:BaselineTokens, 0L)
    $remainingTokens = [math]::Max($effectiveWindow - $adjustedUsed, 0L)
    $percentRemaining = [int][math]::Floor((($remainingTokens / [double]$effectiveWindow) * 100.0) + 0.5)
    $eventTime = [datetime]::Parse(
        [string]$event.timestamp,
        [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::AdjustToUniversal
    )
    $ageSeconds = [math]::Max(0, [int](($Now.ToUniversalTime() - $eventTime.ToUniversalTime()).TotalSeconds))
    $compacted = $false
    if ($null -ne $PreviousUsedTokens) {
        $compacted = $usedTokens -lt [long]$PreviousUsedTokens
    }

    [pscustomobject]@{
        UsedTokens        = $usedTokens
        ContextWindow     = $contextWindow
        EffectiveWindow   = $effectiveWindow
        RemainingTokens   = $remainingTokens
        PercentRemaining  = $percentRemaining
        EventTimestampUtc = $eventTime.ToUniversalTime().ToString('o')
        AgeSeconds        = $ageSeconds
        IsStale           = $ageSeconds -gt $StaleAfterSeconds
        WasCompacted      = $compacted
    }
}

function Get-ContextRolloutMetadata {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)][string] $Path)

    try {
        $firstLine = Get-Content -LiteralPath $Path -TotalCount 1 -ErrorAction Stop
        $record = $firstLine | ConvertFrom-Json -ErrorAction Stop
        if ($record.type -ne 'session_meta' -or $record.payload.originator -ne 'Codex Desktop') {
            return $null
        }

        [pscustomobject]@{
            Path         = $Path
            SessionId    = [string]$record.payload.id
            ThreadSource = [string]$record.payload.thread_source
            Originator   = [string]$record.payload.originator
            LastWriteUtc = (Get-Item -LiteralPath $Path).LastWriteTimeUtc
        }
    }
    catch {
        return $null
    }
}

function Select-ContextRollout {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string] $SessionsRoot,
        [string] $ThreadId,
        [int] $AmbiguousWithinSeconds = 15
    )

    if (-not (Test-Path -LiteralPath $SessionsRoot -PathType Container)) {
        throw "Codex sessions root does not exist: $SessionsRoot"
    }

    $files = Get-ChildItem -LiteralPath $SessionsRoot -Filter 'rollout-*.jsonl' -File -Recurse |
        Sort-Object LastWriteTimeUtc -Descending

    $candidates = New-Object System.Collections.Generic.List[object]
    foreach ($file in $files) {
        if ($ThreadId -and $file.Name -notlike "*$ThreadId*") {
            continue
        }

        $metadata = Get-ContextRolloutMetadata -Path $file.FullName
        if ($null -eq $metadata) {
            continue
        }
        if (-not $ThreadId -and $metadata.ThreadSource -eq 'subagent') {
            continue
        }
        if ($ThreadId -and $metadata.SessionId -ne $ThreadId) {
            continue
        }

        $candidates.Add($metadata)
        if ($ThreadId -or $candidates.Count -ge 2) {
            break
        }
    }

    if ($candidates.Count -eq 0) {
        throw 'No matching root Codex Desktop rollout was found.'
    }

    $selected = $candidates[0]
    $ambiguous = $false
    if (-not $ThreadId -and $candidates.Count -gt 1) {
        $ageGap = [math]::Abs(($selected.LastWriteUtc - $candidates[1].LastWriteUtc).TotalSeconds)
        $ambiguous = $ageGap -le $AmbiguousWithinSeconds
    }

    [pscustomobject]@{
        Path      = $selected.Path
        SessionId = $selected.SessionId
        ShortId   = $selected.SessionId.Substring([math]::Max(0, $selected.SessionId.Length - 8))
        Ambiguous = $ambiguous
    }
}

function Get-ContextOverlayState {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string] $RolloutPath,
        [int] $StaleAfterSeconds = 300,
        [Nullable[long]] $PreviousUsedTokens
    )

    $metadata = Get-ContextRolloutMetadata -Path $RolloutPath
    if ($null -eq $metadata) {
        throw 'The selected file is not a valid Codex Desktop rollout.'
    }

    $tokens = $null
    foreach ($tailBytes in 262144, 1048576, 4194304, 16777216) {
        $tail = @(Read-ContextRolloutTail -Path $RolloutPath -MaximumBytes $tailBytes)
        try {
            $tokens = ConvertFrom-ContextTokenEvent -Lines $tail -StaleAfterSeconds $StaleAfterSeconds -PreviousUsedTokens $PreviousUsedTokens
            break
        }
        catch {
            if ($tailBytes -eq 16777216) { throw }
        }
    }
    [pscustomobject]@{
        SessionId        = $metadata.SessionId
        ShortId          = $metadata.SessionId.Substring([math]::Max(0, $metadata.SessionId.Length - 8))
        SourcePath       = $metadata.Path
        UsedTokens       = $tokens.UsedTokens
        ContextWindow    = $tokens.ContextWindow
        EffectiveWindow  = $tokens.EffectiveWindow
        RemainingTokens  = $tokens.RemainingTokens
        PercentRemaining = $tokens.PercentRemaining
        EventTimestampUtc = $tokens.EventTimestampUtc
        AgeSeconds       = $tokens.AgeSeconds
        IsStale          = $tokens.IsStale
        WasCompacted     = $tokens.WasCompacted
    }
}

function Read-ContextRolloutTail {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string] $Path,
        [int] $MaximumBytes = 1048576
    )

    $stream = New-Object IO.FileStream(
        $Path,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        ([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)
    )
    try {
        $count = [int][math]::Min([long]$MaximumBytes, $stream.Length)
        $start = $stream.Length - $count
        [void]$stream.Seek($start, [IO.SeekOrigin]::Begin)
        $buffer = New-Object byte[] $count
        $read = 0
        while ($read -lt $count) {
            $chunk = $stream.Read($buffer, $read, ($count - $read))
            if ($chunk -eq 0) { break }
            $read += $chunk
        }
        $text = [Text.Encoding]::UTF8.GetString($buffer, 0, $read)
        $lines = @($text -split "\r?\n")
        if ($start -gt 0 -and $lines.Count -gt 0) {
            $lines = @($lines | Select-Object -Skip 1)
        }
        @($lines | Where-Object { $_.Length -gt 0 })
    }
    finally {
        $stream.Dispose()
    }
}

function Initialize-ContextOverlayNativeApi {
    if ('ContextOverlay.NativeMethods' -as [type]) {
        return
    }

    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

namespace ContextOverlay {
    public sealed class WindowInfo {
        public IntPtr Handle { get; set; }
        public int ProcessId { get; set; }
        public string Title { get; set; }
        public string ClassName { get; set; }
        public int Left { get; set; }
        public int Top { get; set; }
        public int Right { get; set; }
        public int Bottom { get; set; }
        public bool IsMinimized { get; set; }
    }

    public static class NativeMethods {
        public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

        [StructLayout(LayoutKind.Sequential)]
        public struct RECT { public int Left, Top, Right, Bottom; }

        [StructLayout(LayoutKind.Sequential)]
        public struct POINT { public int X, Y; }

        [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr extraData);
        [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
        [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
        [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
        [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr hWnd, StringBuilder text, int count);
        [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll", EntryPoint = "GetWindowLongPtr")] public static extern IntPtr GetWindowLongPtr64(IntPtr hWnd, int index);
        [DllImport("user32.dll", EntryPoint = "GetWindowLong")] public static extern IntPtr GetWindowLongPtr32(IntPtr hWnd, int index);
        [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr")] public static extern IntPtr SetWindowLongPtr64(IntPtr hWnd, int index, IntPtr value);
        [DllImport("user32.dll", EntryPoint = "SetWindowLong")] public static extern IntPtr SetWindowLongPtr32(IntPtr hWnd, int index, IntPtr value);
        [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT point);
        [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr hWnd);
        [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);
        [DllImport("gdi32.dll")] public static extern uint GetPixel(IntPtr hDC, int x, int y);
        [DllImport("gdi32.dll")] public static extern IntPtr CreateEllipticRgn(int left, int top, int right, int bottom);
        [DllImport("user32.dll")] public static extern int SetWindowRgn(IntPtr hWnd, IntPtr region, bool redraw);

        public static IntPtr GetWindowLongPtr(IntPtr hWnd, int index) {
            return IntPtr.Size == 8 ? GetWindowLongPtr64(hWnd, index) : GetWindowLongPtr32(hWnd, index);
        }

        public static IntPtr SetWindowLongPtr(IntPtr hWnd, int index, IntPtr value) {
            return IntPtr.Size == 8 ? SetWindowLongPtr64(hWnd, index, value) : SetWindowLongPtr32(hWnd, index, value);
        }

        public static List<WindowInfo> EnumerateVisibleWindows() {
            var windows = new List<WindowInfo>();
            EnumWindows(delegate(IntPtr handle, IntPtr unused) {
                if (!IsWindowVisible(handle)) return true;
                RECT rect;
                if (!GetWindowRect(handle, out rect)) return true;
                if (rect.Right - rect.Left < 300 || rect.Bottom - rect.Top < 200) return true;
                uint processId;
                GetWindowThreadProcessId(handle, out processId);
                var title = new StringBuilder(512);
                var className = new StringBuilder(256);
                GetWindowText(handle, title, title.Capacity);
                GetClassName(handle, className, className.Capacity);
                windows.Add(new WindowInfo {
                    Handle = handle,
                    ProcessId = (int)processId,
                    Title = title.ToString(),
                    ClassName = className.ToString(),
                    Left = rect.Left,
                    Top = rect.Top,
                    Right = rect.Right,
                    Bottom = rect.Bottom,
                    IsMinimized = IsIconic(handle)
                });
                return true;
            }, IntPtr.Zero);
            return windows;
        }
    }
}
'@
}

function Get-CodexWindowAnchor {
    [CmdletBinding()]
    param(
        [int] $DialWidth = 190,
        [int] $DialHeight = 28,
        [int] $RightOffset = 152,
        [int] $BottomOffset = 104
    )

    Initialize-ContextOverlayNativeApi
    $codexProcessIds = @(Get-Process -Name ChatGPT -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -like '*OpenAI.Codex_*' } |
        Select-Object -ExpandProperty Id)
    if ($codexProcessIds.Count -eq 0) {
        throw 'No installed Codex Desktop process was found.'
    }

    $windows = @([ContextOverlay.NativeMethods]::EnumerateVisibleWindows() |
        Where-Object { $codexProcessIds -contains $_.ProcessId } |
        Sort-Object @{ Expression = { ($_.Right - $_.Left) * ($_.Bottom - $_.Top) }; Descending = $true })
    if ($windows.Count -eq 0) {
        throw 'No visible Codex Desktop top-level window was found.'
    }

    $window = $windows[0]
    $foreground = [ContextOverlay.NativeMethods]::GetForegroundWindow()
    [uint32]$foregroundProcessId = 0
    [void][ContextOverlay.NativeMethods]::GetWindowThreadProcessId($foreground, [ref]$foregroundProcessId)
    $left = $window.Right - $RightOffset - $DialWidth
    $top = $window.Bottom - $BottomOffset - $DialHeight

    [pscustomobject]@{
        WindowHandle = ('0x{0:X}' -f $window.Handle.ToInt64())
        ProcessId    = $window.ProcessId
        Title        = $window.Title
        ClassName    = $window.ClassName
        WindowLeft   = $window.Left
        WindowTop    = $window.Top
        WindowRight  = $window.Right
        WindowBottom = $window.Bottom
        IsMinimized  = $window.IsMinimized
        IsForeground = $codexProcessIds -contains [int]$foregroundProcessId
        AnchorLeft   = $left
        AnchorTop    = $top
        DialWidth    = $DialWidth
        DialHeight   = $DialHeight
    }
}

function Get-CodexPromptPalette {
    [CmdletBinding()]
    param([Parameter(Mandatory = $true)] $Window)

    Initialize-ContextOverlayNativeApi
    $deviceContext = [ContextOverlay.NativeMethods]::GetDC([IntPtr]::Zero)
    if ($deviceContext -eq [IntPtr]::Zero) {
        throw 'Could not acquire the screen device context for prompt-pill color sampling.'
    }
    try {
        $samples = New-Object System.Collections.Generic.List[object]
        foreach ($rightInset in 700, 800, 900) {
            $color = [ContextOverlay.NativeMethods]::GetPixel(
                $deviceContext,
                ($Window.WindowRight - $rightInset),
                ($Window.WindowBottom - 110)
            )
            if ($color -ne [uint32]::MaxValue) {
                $samples.Add([pscustomobject]@{
                    R = [int]($color -band 0xFF)
                    G = [int](($color -shr 8) -band 0xFF)
                    B = [int](($color -shr 16) -band 0xFF)
                })
            }
        }
        if ($samples.Count -eq 0) {
            throw 'Prompt-pill color sampling returned no valid pixels.'
        }
        $middle = [int][math]::Floor($samples.Count / 2)
        $red = [int](@($samples.R | Sort-Object)[$middle])
        $green = [int](@($samples.G | Sort-Object)[$middle])
        $blue = [int](@($samples.B | Sort-Object)[$middle])
        $luminance = (0.2126 * $red) + (0.7152 * $green) + (0.0722 * $blue)
        $foreground = if ($luminance -gt 145) { 32 } else { 246 }
        [pscustomobject]@{
            BackgroundR = $red
            BackgroundG = $green
            BackgroundB = $blue
            ForegroundR = $foreground
            ForegroundG = $foreground
            ForegroundB = $foreground
            MutedR = [int][math]::Round(($red * 0.35) + ($foreground * 0.65))
            MutedG = [int][math]::Round(($green * 0.35) + ($foreground * 0.65))
            MutedB = [int][math]::Round(($blue * 0.35) + ($foreground * 0.65))
            TrackR = [int][math]::Round(($red * 0.72) + ($foreground * 0.28))
            TrackG = [int][math]::Round(($green * 0.72) + ($foreground * 0.28))
            TrackB = [int][math]::Round(($blue * 0.72) + ($foreground * 0.28))
            BorderR = [int][math]::Round(($red * 0.84) + ($foreground * 0.16))
            BorderG = [int][math]::Round(($green * 0.84) + ($foreground * 0.16))
            BorderB = [int][math]::Round(($blue * 0.84) + ($foreground * 0.16))
            IsLight = $luminance -gt 145
        }
    }
    finally {
        [void][ContextOverlay.NativeMethods]::ReleaseDC([IntPtr]::Zero, $deviceContext)
    }
}

Export-ModuleMember -Function ConvertFrom-ContextTokenEvent, Get-ContextRolloutMetadata, Select-ContextRollout, Get-ContextOverlayState, Read-ContextRolloutTail, Get-CodexWindowAnchor, Get-CodexPromptPalette
