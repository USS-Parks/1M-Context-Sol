using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;

namespace OneMContextTicker
{
    internal sealed class CodexWindowInfo
    {
        public IntPtr Handle { get; set; }
        public int ProcessId { get; set; }
        public string Title { get; set; }
        public string ClassName { get; set; }
        public int Left { get; set; }
        public int Top { get; set; }
        public int Right { get; set; }
        public int Bottom { get; set; }
        public bool IsMinimized { get; set; }
        public bool IsForeground { get; set; }
    }

    internal sealed class PromptPalette
    {
        public byte BackgroundR { get; set; }
        public byte BackgroundG { get; set; }
        public byte BackgroundB { get; set; }
        public byte MutedR { get; set; }
        public byte MutedG { get; set; }
        public byte MutedB { get; set; }
        public int PromptCenter { get; set; }
        public bool SidebarOpen { get; set; }
        public bool IsLight { get; set; }
    }

    internal static class Native
    {
        internal const int GwlExStyle = -20;
        internal const long WsExToolWindow = 0x00000080L;
        internal const long WsExNoActivate = 0x08000000L;
        internal const long WsExTransparent = 0x00000020L;

        internal delegate bool EnumWindowsProc(IntPtr handle, IntPtr extraData);

        [StructLayout(LayoutKind.Sequential)]
        internal struct Rect { public int Left, Top, Right, Bottom; }

        [StructLayout(LayoutKind.Sequential)]
        internal struct Point { public int X, Y; }

        [DllImport("user32.dll")]
        internal static extern bool EnumWindows(EnumWindowsProc callback, IntPtr extraData);
        [DllImport("user32.dll")]
        internal static extern bool IsWindowVisible(IntPtr handle);
        [DllImport("user32.dll")]
        internal static extern bool IsIconic(IntPtr handle);
        [DllImport("user32.dll")]
        internal static extern bool GetWindowRect(IntPtr handle, out Rect rect);
        [DllImport("user32.dll")]
        internal static extern uint GetWindowThreadProcessId(IntPtr handle, out uint processId);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        internal static extern int GetWindowText(IntPtr handle, StringBuilder text, int count);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        internal static extern int GetClassName(IntPtr handle, StringBuilder text, int count);
        [DllImport("user32.dll")]
        internal static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll", EntryPoint = "GetWindowLongPtr")]
        private static extern IntPtr GetWindowLongPtr64(IntPtr handle, int index);
        [DllImport("user32.dll", EntryPoint = "GetWindowLong")]
        private static extern IntPtr GetWindowLongPtr32(IntPtr handle, int index);
        [DllImport("user32.dll", EntryPoint = "SetWindowLongPtr")]
        private static extern IntPtr SetWindowLongPtr64(IntPtr handle, int index, IntPtr value);
        [DllImport("user32.dll", EntryPoint = "SetWindowLong")]
        private static extern IntPtr SetWindowLongPtr32(IntPtr handle, int index, IntPtr value);
        [DllImport("user32.dll")]
        internal static extern IntPtr WindowFromPoint(Point point);
        [DllImport("user32.dll")]
        private static extern IntPtr GetDC(IntPtr handle);
        [DllImport("user32.dll")]
        private static extern int ReleaseDC(IntPtr handle, IntPtr deviceContext);
        [DllImport("gdi32.dll")]
        private static extern uint GetPixel(IntPtr deviceContext, int x, int y);
        [DllImport("gdi32.dll")]
        internal static extern IntPtr CreateRoundRectRgn(int left, int top, int right, int bottom, int ellipseWidth, int ellipseHeight);
        [DllImport("user32.dll")]
        internal static extern int SetWindowRgn(IntPtr handle, IntPtr region, bool redraw);

        internal static IntPtr GetWindowLongPtr(IntPtr handle, int index)
        {
            return IntPtr.Size == 8 ? GetWindowLongPtr64(handle, index) : GetWindowLongPtr32(handle, index);
        }

        internal static IntPtr SetWindowLongPtr(IntPtr handle, int index, IntPtr value)
        {
            return IntPtr.Size == 8 ? SetWindowLongPtr64(handle, index, value) : SetWindowLongPtr32(handle, index, value);
        }

        internal static CodexWindowInfo FindCodexWindow()
        {
            HashSet<int> processIds = new HashSet<int>();
            foreach (Process process in Process.GetProcessesByName("ChatGPT"))
            {
                try
                {
                    string path = process.MainModule == null ? String.Empty : process.MainModule.FileName;
                    if (path.IndexOf("OpenAI.Codex_", StringComparison.OrdinalIgnoreCase) >= 0) processIds.Add(process.Id);
                }
                catch { }
            }
            if (processIds.Count == 0) throw new InvalidOperationException("No installed Codex Desktop process was found.");

            List<CodexWindowInfo> windows = new List<CodexWindowInfo>();
            EnumWindows(delegate(IntPtr handle, IntPtr ignored)
            {
                if (!IsWindowVisible(handle)) return true;
                Rect rect;
                if (!GetWindowRect(handle, out rect)) return true;
                if (rect.Right - rect.Left < 300 || rect.Bottom - rect.Top < 200) return true;
                uint processId;
                GetWindowThreadProcessId(handle, out processId);
                if (!processIds.Contains((int)processId)) return true;
                StringBuilder title = new StringBuilder(512);
                StringBuilder className = new StringBuilder(256);
                GetWindowText(handle, title, title.Capacity);
                GetClassName(handle, className, className.Capacity);
                windows.Add(new CodexWindowInfo
                {
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
            if (windows.Count == 0) throw new InvalidOperationException("No visible Codex Desktop top-level window was found.");

            CodexWindowInfo selected = windows.OrderByDescending(delegate(CodexWindowInfo window)
            {
                return (long)(window.Right - window.Left) * (window.Bottom - window.Top);
            }).First();
            IntPtr foreground = GetForegroundWindow();
            uint foregroundProcessId;
            GetWindowThreadProcessId(foreground, out foregroundProcessId);
            selected.IsForeground = processIds.Contains((int)foregroundProcessId);
            return selected;
        }

        internal static PromptPalette ReadPromptPalette(CodexWindowInfo window)
        {
            IntPtr deviceContext = GetDC(IntPtr.Zero);
            if (deviceContext == IntPtr.Zero) throw new InvalidOperationException("Could not acquire screen device context.");
            try
            {
                int[] red = new int[3];
                int[] green = new int[3];
                int[] blue = new int[3];
                int[] insets = { 700, 800, 900 };
                for (int index = 0; index < insets.Length; index++)
                {
                    uint color = GetPixel(deviceContext, window.Right - insets[index], window.Bottom - 110);
                    red[index] = (int)(color & 0xFF);
                    green[index] = (int)((color >> 8) & 0xFF);
                    blue[index] = (int)((color >> 16) & 0xFF);
                }
                Array.Sort(red);
                Array.Sort(green);
                Array.Sort(blue);
                int backgroundRed = red[1];
                int backgroundGreen = green[1];
                int backgroundBlue = blue[1];
                int width = window.Right - window.Left;
                int height = window.Bottom - window.Top;
                uint sidebarColor = GetPixel(deviceContext, window.Right - (int)(width * 0.10), window.Top + (int)(height * 0.25));
                int sidebarRed = (int)(sidebarColor & 0xFF);
                int sidebarGreen = (int)((sidebarColor >> 8) & 0xFF);
                int sidebarBlue = (int)((sidebarColor >> 16) & 0xFF);
                bool sidebarOpen = Math.Abs(sidebarRed - backgroundRed) <= 12
                    && Math.Abs(sidebarGreen - backgroundGreen) <= 12
                    && Math.Abs(sidebarBlue - backgroundBlue) <= 12;
                double luminance = (0.2126 * backgroundRed) + (0.7152 * backgroundGreen) + (0.0722 * backgroundBlue);
                int foreground = luminance > 145 ? 32 : 246;
                return new PromptPalette
                {
                    BackgroundR = (byte)backgroundRed,
                    BackgroundG = (byte)backgroundGreen,
                    BackgroundB = (byte)backgroundBlue,
                    MutedR = (byte)Math.Round((backgroundRed * 0.35) + (foreground * 0.65)),
                    MutedG = (byte)Math.Round((backgroundGreen * 0.35) + (foreground * 0.65)),
                    MutedB = (byte)Math.Round((backgroundBlue * 0.35) + (foreground * 0.65)),
                    PromptCenter = ComputeComposerCenter(window.Left, window.Right, sidebarOpen),
                    SidebarOpen = sidebarOpen,
                    IsLight = luminance > 145
                };
            }
            finally
            {
                ReleaseDC(IntPtr.Zero, deviceContext);
            }
        }

        internal static int ComputeComposerCenter(int windowLeft, int windowRight, bool sidebarOpen)
        {
            int width = windowRight - windowLeft;
            if (width <= 0) throw new ArgumentException("windowRight must be greater than windowLeft.");
            double navigationWidth = width * 0.15625;
            double sidebarWidth = sidebarOpen ? width * 0.203125 : 0.0;
            return (int)Math.Round(windowLeft + navigationWidth + ((width - navigationWidth - sidebarWidth) / 2.0));
        }
    }
}
