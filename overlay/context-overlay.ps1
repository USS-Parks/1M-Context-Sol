[CmdletBinding()]
param(
    [switch] $DryRun,
    [string] $SessionsRoot = (Join-Path $env:USERPROFILE '.codex\sessions'),
    [string] $ThreadId,
    [int] $StaleAfterSeconds = 300,
    [int] $RightOffset = 935,
    [int] $BottomOffset = 42,
    [int] $CompactionThreshold = 900000,
    [int] $RefreshMilliseconds = 1000,
    [string] $OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'ContextOverlay.Core.psm1') -Force

$createdNew = $false
$script:singleInstanceMutex = New-Object Threading.Mutex($true, 'Local\CodexContextOverlay', [ref]$createdNew)
if (-not $createdNew) {
    $script:singleInstanceMutex.Dispose()
    return
}

$selection = Select-ContextRollout -SessionsRoot $SessionsRoot -ThreadId $ThreadId
$state = Get-ContextOverlayState -RolloutPath $selection.Path -StaleAfterSeconds $StaleAfterSeconds
$anchor = $null
try {
    $anchor = Get-CodexWindowAnchor -RightOffset $RightOffset -BottomOffset $BottomOffset
}
catch {
    if ($DryRun) { throw }
}

$result = [pscustomobject]@{
    Mode             = if ($DryRun) { 'dry-run' } else { 'overlay' }
    SessionId        = $state.SessionId
    ShortId          = $state.ShortId
    SourcePath       = $state.SourcePath
    SelectionIsAmbiguous = $selection.Ambiguous
    UsedTokens       = $state.UsedTokens
    ContextWindow    = $state.ContextWindow
    EffectiveWindow  = $state.EffectiveWindow
    RemainingTokens  = $state.RemainingTokens
    PercentRemaining = $state.PercentRemaining
    EventTimestampUtc = $state.EventTimestampUtc
    AgeSeconds       = $state.AgeSeconds
    IsStale          = $state.IsStale
    WasCompacted     = $state.WasCompacted
    Window           = $anchor
} | ConvertTo-Json -Depth 5

if ($OutputPath) {
    [IO.File]::WriteAllText($OutputPath, $result, (New-Object Text.UTF8Encoding($false)))
}
else {
    $result
}

if ($DryRun) {
    return
}

Add-Type -AssemblyName PresentationCore, PresentationFramework, WindowsBase

[xml]$xaml = @'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        Title="Codex Context Dial"
        Width="190" Height="28" WindowStyle="None" ResizeMode="NoResize"
        AllowsTransparency="True" Background="Transparent" Topmost="True"
        ShowInTaskbar="False" ShowActivated="False" Focusable="False" Opacity="0">
    <Grid Name="DialRoot" ToolTipService.ShowDuration="60000">
        <Border Name="Capsule" Background="#FF303030" BorderThickness="0" CornerRadius="14" />
        <TextBlock Name="UsedText" Text="Context: -- / 1M"
                   Foreground="#FFF6F7F9" FontFamily="Cascadia Mono, Consolas"
                   FontSize="12" FontWeight="Normal"
                   HorizontalAlignment="Center" VerticalAlignment="Center" />
    </Grid>
</Window>
'@

$reader = New-Object Xml.XmlNodeReader $xaml
$window = [Windows.Markup.XamlReader]::Load($reader)
$dialRoot = $window.FindName('DialRoot')
$capsule = $window.FindName('Capsule')
$usedText = $window.FindName('UsedText')

function New-RgbBrush([int] $Red, [int] $Green, [int] $Blue) {
    New-Object Windows.Media.SolidColorBrush ([Windows.Media.Color]::FromRgb([byte]$Red, [byte]$Green, [byte]$Blue))
}

$script:selectedPath = $selection.Path
$script:selectionAmbiguous = $selection.Ambiguous
$script:previousUsed = $null
$script:lastState = $state
$script:overlayHandle = [IntPtr]::Zero
$script:lastPalette = $null

$window.Add_SourceInitialized({
    $helper = New-Object Windows.Interop.WindowInteropHelper $window
    $handle = $helper.Handle
    $script:overlayHandle = $handle
    $extendedStyle = [ContextOverlay.NativeMethods]::GetWindowLongPtr($handle, -20).ToInt64()
    $extendedStyle = $extendedStyle -bor 0x00000080L -bor 0x08000000L
    [void][ContextOverlay.NativeMethods]::SetWindowLongPtr($handle, -20, [IntPtr]$extendedStyle)
    $rect = New-Object ContextOverlay.NativeMethods+RECT
    if ([ContextOverlay.NativeMethods]::GetWindowRect($handle, [ref]$rect)) {
        $region = [ContextOverlay.NativeMethods]::CreateEllipticRgn(0, 0, ($rect.Right - $rect.Left + 1), ($rect.Bottom - $rect.Top + 1))
        if ($region -ne [IntPtr]::Zero) {
            [void][ContextOverlay.NativeMethods]::SetWindowRgn($handle, $region, $true)
        }
    }
})

$dialRoot.Add_MouseRightButtonUp({ $window.Close() })

function Write-OverlayStatus($CurrentState, $Position, [bool] $Visible, [string] $ErrorMessage) {
    if (-not $OutputPath) {
        return
    }
    $nativeLeft = $null
    $nativeTop = $null
    $nativeWidth = $null
    $nativeHeight = $null
    $hasNoActivate = $false
    $hasToolWindow = $false
    $cornerPassesThrough = $null
    $centerBelongsToDial = $null
    if ($script:overlayHandle -ne [IntPtr]::Zero) {
        $nativeRect = New-Object ContextOverlay.NativeMethods+RECT
        if ([ContextOverlay.NativeMethods]::GetWindowRect($script:overlayHandle, [ref]$nativeRect)) {
            $nativeLeft = $nativeRect.Left
            $nativeTop = $nativeRect.Top
            $nativeWidth = $nativeRect.Right - $nativeRect.Left
            $nativeHeight = $nativeRect.Bottom - $nativeRect.Top
        }
        $style = [ContextOverlay.NativeMethods]::GetWindowLongPtr($script:overlayHandle, -20).ToInt64()
        $hasToolWindow = 0 -ne ($style -band 0x00000080L)
        $hasNoActivate = 0 -ne ($style -band 0x08000000L)
        if ($null -ne $nativeLeft -and $Visible) {
            $cornerPoint = New-Object ContextOverlay.NativeMethods+POINT
            $cornerPoint.X = $nativeLeft + 1
            $cornerPoint.Y = $nativeTop + 1
            $centerPoint = New-Object ContextOverlay.NativeMethods+POINT
            $centerPoint.X = $nativeLeft + [int]($nativeWidth / 2)
            $centerPoint.Y = $nativeTop + [int]($nativeHeight / 2)
            $cornerPassesThrough = [ContextOverlay.NativeMethods]::WindowFromPoint($cornerPoint) -ne $script:overlayHandle
            $centerBelongsToDial = [ContextOverlay.NativeMethods]::WindowFromPoint($centerPoint) -eq $script:overlayHandle
        }
    }
    $status = [pscustomobject]@{
        Mode              = 'overlay'
        ProcessId         = $PID
        OverlayHandle     = if ($script:overlayHandle -eq [IntPtr]::Zero) { $null } else { '0x{0:X}' -f $script:overlayHandle.ToInt64() }
        Visible           = $Visible
        NativeLeft        = $nativeLeft
        NativeTop         = $nativeTop
        NativeWidth       = $nativeWidth
        NativeHeight      = $nativeHeight
        HasToolWindow     = $hasToolWindow
        HasNoActivate     = $hasNoActivate
        CornerPassesThrough = $cornerPassesThrough
        CenterBelongsToDial = $centerBelongsToDial
        SessionId         = if ($null -eq $CurrentState) { $null } else { $CurrentState.SessionId }
        UsedTokens        = if ($null -eq $CurrentState) { $null } else { $CurrentState.UsedTokens }
        ContextWindow     = if ($null -eq $CurrentState) { $null } else { $CurrentState.ContextWindow }
        RemainingTokens   = if ($null -eq $CurrentState) { $null } else { $CurrentState.RemainingTokens }
        PercentRemaining  = if ($null -eq $CurrentState) { $null } else { $CurrentState.PercentRemaining }
        PercentUsed       = if ($null -eq $CurrentState) { $null } else { 100 - $CurrentState.PercentRemaining }
        IsStale           = if ($null -eq $CurrentState) { $null } else { $CurrentState.IsStale }
        SelectionAmbiguous = $script:selectionAmbiguous
        PromptBackground  = if ($null -eq $script:lastPalette) { $null } else { '#{0:X2}{1:X2}{2:X2}' -f $script:lastPalette.BackgroundR, $script:lastPalette.BackgroundG, $script:lastPalette.BackgroundB }
        PromptTheme       = if ($null -eq $script:lastPalette) { $null } elseif ($script:lastPalette.IsLight) { 'light' } else { 'dark' }
        CompactionThreshold = $CompactionThreshold
        AnchorLeftDip     = if ($null -eq $Position) { $null } else { [math]::Round($window.Left, 1) }
        AnchorTopDip      = if ($null -eq $Position) { $null } else { [math]::Round($window.Top, 1) }
        AnchorLeftDevice  = if ($null -eq $Position) { $null } else { $Position.AnchorLeft }
        AnchorTopDevice   = if ($null -eq $Position) { $null } else { $Position.AnchorTop }
        CodexForeground   = if ($null -eq $Position) { $false } else { $Position.IsForeground }
        CodexMinimized    = if ($null -eq $Position) { $null } else { $Position.IsMinimized }
        Error             = $ErrorMessage
        UpdatedUtc        = [datetime]::UtcNow.ToString('o')
    } | ConvertTo-Json -Depth 4
    [IO.File]::WriteAllText($OutputPath, $status, (New-Object Text.UTF8Encoding($false)))
}

function Update-Overlay {
    try {
        if (-not $ThreadId) {
            $candidate = Select-ContextRollout -SessionsRoot $SessionsRoot
            if ($candidate.Path -ne $script:selectedPath) {
                $selectedWrite = (Get-Item -LiteralPath $script:selectedPath -ErrorAction SilentlyContinue).LastWriteTimeUtc
                $candidateWrite = (Get-Item -LiteralPath $candidate.Path).LastWriteTimeUtc
                if ($null -eq $selectedWrite -or $candidateWrite -gt $selectedWrite.AddSeconds(3)) {
                    $script:selectedPath = $candidate.Path
                    $script:previousUsed = $null
                }
            }
            $script:selectionAmbiguous = $candidate.Ambiguous
        }

        $current = Get-ContextOverlayState -RolloutPath $script:selectedPath -StaleAfterSeconds $StaleAfterSeconds -PreviousUsedTokens $script:previousUsed
        $script:previousUsed = $current.UsedTokens
        $script:lastState = $current
        $position = Get-CodexWindowAnchor -RightOffset $RightOffset -BottomOffset $BottomOffset

        if ($position.IsMinimized -or -not $position.IsForeground) {
            $window.Hide()
            Write-OverlayStatus $current $position $false $null
            return
        }

        $devicePoint = New-Object Windows.Point($position.AnchorLeft, $position.AnchorTop)
        $presentationSource = [Windows.PresentationSource]::FromVisual($window)
        if ($null -ne $presentationSource -and $null -ne $presentationSource.CompositionTarget) {
            $dipPoint = $presentationSource.CompositionTarget.TransformFromDevice.Transform($devicePoint)
            $window.Left = $dipPoint.X
        $window.Top = $dipPoint.Y
        }
        else {
            $window.Left = $position.AnchorLeft
            $window.Top = $position.AnchorTop
        }
        $usedPercent = 100 - $current.PercentRemaining
        $usedText.Text = ('Context: {0:N0} / 1M' -f $current.UsedTokens)
        $palette = Get-CodexPromptPalette -Window $position
        $script:lastPalette = $palette
        $capsule.Background = New-RgbBrush $palette.BackgroundR $palette.BackgroundG $palette.BackgroundB
        $usedText.Foreground = New-RgbBrush $palette.MutedR $palette.MutedG $palette.MutedB

        if ($current.IsStale) {
            $usedText.Foreground = [Windows.Media.Brushes]::SlateGray
        }
        elseif ($script:selectionAmbiguous) {
            $usedText.Foreground = [Windows.Media.Brushes]::Gold
        }
        elseif ($usedPercent -ge 90) {
            $usedText.Foreground = [Windows.Media.Brushes]::OrangeRed
        }
        elseif ($usedPercent -ge 75) {
            $usedText.Foreground = [Windows.Media.Brushes]::Orange
        }

        $status = if ($current.IsStale) { 'STALE' } elseif ($script:selectionAmbiguous) { 'AMBIGUOUS' } else { 'LIVE' }
        $dialRoot.ToolTip = @"
$status context state
Task: $($current.SessionId)
Active context: $($current.UsedTokens)
Remaining: $($current.RemainingTokens) of $($current.EffectiveWindow) effective tokens
Host window: $($current.ContextWindow)
Automatic compaction threshold: $CompactionThreshold total tokens
Source: $($current.SourcePath)
Right-click the dial to stop the overlay.
"@
        if (-not $window.IsVisible) {
            $window.Show()
        }
        $window.Opacity = 1
        Write-OverlayStatus $current $position $true $null
    }
    catch {
        $usedText.Text = 'Context: ! / 1M'
        $usedText.Foreground = [Windows.Media.Brushes]::OrangeRed
        $dialRoot.ToolTip = 'Context overlay error: ' + $_.Exception.Message
        try {
            $position = Get-CodexWindowAnchor -RightOffset $RightOffset -BottomOffset $BottomOffset
            if ($position.IsForeground -and -not $position.IsMinimized) {
                $devicePoint = New-Object Windows.Point($position.AnchorLeft, $position.AnchorTop)
                $presentationSource = [Windows.PresentationSource]::FromVisual($window)
                if ($null -ne $presentationSource -and $null -ne $presentationSource.CompositionTarget) {
                    $dipPoint = $presentationSource.CompositionTarget.TransformFromDevice.Transform($devicePoint)
                    $window.Left = $dipPoint.X
                    $window.Top = $dipPoint.Y
                }
                if (-not $window.IsVisible) { $window.Show() }
            }
            else {
                $window.Hide()
            }
            Write-OverlayStatus $script:lastState $position $window.IsVisible $_.Exception.Message
        }
        catch {
            $window.Hide()
            Write-OverlayStatus $script:lastState $null $false $_.Exception.Message
        }
    }
}

$timer = New-Object Windows.Threading.DispatcherTimer
$timer.Interval = [timespan]::FromMilliseconds([math]::Max(250, $RefreshMilliseconds))
$timer.Add_Tick({ Update-Overlay })
$window.Add_Closed({
    $timer.Stop()
    if ($null -ne $script:singleInstanceMutex) {
        $script:singleInstanceMutex.ReleaseMutex()
        $script:singleInstanceMutex.Dispose()
        $script:singleInstanceMutex = $null
    }
    $window.Dispatcher.InvokeShutdown()
})
$window.Add_Loaded({
    Update-Overlay
    $timer.Start()
})
$window.Show()
[Windows.Threading.Dispatcher]::Run()
