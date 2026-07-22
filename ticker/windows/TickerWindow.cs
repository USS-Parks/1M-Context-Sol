using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Threading;
using System.Web.Script.Serialization;

namespace OneMContextTicker
{
    internal sealed class TickerWindow : Window
    {
        private const double FaceFontSize = 12.0;
        private const double FaceHorizontalPadding = 5.0;
        private const double FaceWidthSafetyMargin = 12.0;
        private static readonly FontFamily FaceFontFamily = new FontFamily("Cascadia Mono, Consolas");
        private readonly Options options;
        private readonly Border capsule;
        private readonly TextBlock text;
        private readonly DispatcherTimer timer;
        private string selectedPath;
        private bool selectionAmbiguous;
        private long? previousUsed;
        private TokenState lastState;
        private PromptPalette lastPalette;
        private IntPtr handle;

        public TickerWindow(Options options)
        {
            this.options = options;
            Title = "1M Context Ticker";
            SizeToContent = SizeToContent.Height;
            WindowStyle = WindowStyle.None;
            ResizeMode = ResizeMode.NoResize;
            AllowsTransparency = true;
            Background = Brushes.Transparent;
            Topmost = true;
            ShowInTaskbar = false;
            ShowActivated = false;
            Focusable = false;
            Opacity = 0;

            text = new TextBlock
            {
                Text = "Context: -- / 1M",
                FontFamily = FaceFontFamily,
                FontSize = FaceFontSize,
                FontWeight = FontWeights.Normal,
                Foreground = Brushes.White,
                TextWrapping = TextWrapping.NoWrap
            };
            capsule = new Border
            {
                Background = new SolidColorBrush(Color.FromRgb(48, 48, 48)),
                BorderThickness = new Thickness(0),
                CornerRadius = new CornerRadius(13),
                Padding = new Thickness(FaceHorizontalPadding),
                Child = text
            };
            Content = capsule;
            Width = RequiredFaceWidth(text.Text);

            SourceInitialized += OnSourceInitialized;
            SizeChanged += delegate { UpdateRegion(); };
            Loaded += OnLoaded;
            Closed += OnClosed;

            timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(1000) };
            timer.Tick += delegate { UpdateTicker(); };
        }

        private void OnSourceInitialized(object sender, EventArgs args)
        {
            handle = new WindowInteropHelper(this).Handle;
            long style = Native.GetWindowLongPtr(handle, Native.GwlExStyle).ToInt64();
            style |= Native.WsExToolWindow | Native.WsExNoActivate | Native.WsExTransparent;
            Native.SetWindowLongPtr(handle, Native.GwlExStyle, new IntPtr(style));
            UpdateRegion();
        }

        private void OnLoaded(object sender, RoutedEventArgs args)
        {
            UpdateTicker();
            timer.Start();
        }

        private void OnClosed(object sender, EventArgs args)
        {
            timer.Stop();
            if (Application.Current != null) Application.Current.Shutdown();
        }

        private void UpdateRegion()
        {
            if (handle == IntPtr.Zero) return;
            Native.Rect rect;
            if (!Native.GetWindowRect(handle, out rect)) return;
            int width = rect.Right - rect.Left;
            int height = rect.Bottom - rect.Top;
            IntPtr region = Native.CreateRoundRectRgn(0, 0, width + 1, height + 1, height, height);
            if (region != IntPtr.Zero) Native.SetWindowRgn(handle, region, true);
        }

        private void UpdateTicker()
        {
            CodexWindowInfo codexWindow = null;
            try
            {
                RolloutSelection candidate = RolloutSelector.Select(options.SessionsRoot, options.ThreadId, 15);
                if (String.IsNullOrEmpty(selectedPath)) selectedPath = candidate.Path;
                else if (candidate.Path != selectedPath)
                {
                    DateTime selectedWrite = File.Exists(selectedPath) ? File.GetLastWriteTimeUtc(selectedPath) : DateTime.MinValue;
                    DateTime candidateWrite = File.GetLastWriteTimeUtc(candidate.Path);
                    if (candidateWrite > selectedWrite.AddSeconds(3))
                    {
                        selectedPath = candidate.Path;
                        previousUsed = null;
                    }
                }
                selectionAmbiguous = candidate.Ambiguous;
                TokenState state = RolloutReader.ReadState(selectedPath, options.StaleAfterSeconds, previousUsed);
                previousUsed = state.UsedTokens;
                lastState = state;
                codexWindow = Native.FindCodexWindow();

                if (codexWindow.IsMinimized || !codexWindow.IsForeground)
                {
                    Hide();
                    WriteStatus(state, codexWindow, false, null);
                    return;
                }

                PromptPalette palette = Native.ReadPromptPalette(codexWindow);
                lastPalette = palette;
                text.Text = String.Format(CultureInfo.CurrentCulture, "Context: {0:N0} / 1M", state.UsedTokens);
                Width = RequiredFaceWidth(text.Text);
                capsule.Background = Brush(palette.BackgroundR, palette.BackgroundG, palette.BackgroundB);
                text.Foreground = Brush(palette.MutedR, palette.MutedG, palette.MutedB);
                int percentUsed = 100 - state.PercentRemaining;
                if (state.IsStale) text.Foreground = Brushes.SlateGray;
                else if (selectionAmbiguous) text.Foreground = Brushes.Gold;
                else if (percentUsed >= 90) text.Foreground = Brushes.OrangeRed;
                else if (percentUsed >= 75) text.Foreground = Brushes.Orange;

                UpdateLayout();
                UpdateRegion();
                SetLocation(codexWindow, palette);
                if (!IsVisible) Show();
                Opacity = 1;
                WriteStatus(state, codexWindow, true, null);
            }
            catch (Exception error)
            {
                text.Text = "Context: !";
                Width = RequiredFaceWidth(text.Text);
                text.Foreground = Brushes.OrangeRed;
                try
                {
                    codexWindow = codexWindow ?? Native.FindCodexWindow();
                    if (codexWindow.IsForeground && !codexWindow.IsMinimized)
                    {
                        PromptPalette palette = lastPalette ?? Native.ReadPromptPalette(codexWindow);
                        lastPalette = palette;
                        SetLocation(codexWindow, palette);
                        if (!IsVisible) Show();
                        Opacity = 1;
                    }
                    else Hide();
                }
                catch { Hide(); }
                WriteStatus(lastState, codexWindow, IsVisible, error.Message);
            }
        }

        private void SetLocation(CodexWindowInfo codexWindow, PromptPalette palette)
        {
            Point deviceCenter = new Point(palette.PromptCenter, codexWindow.Bottom - 53.0);
            PresentationSource source = PresentationSource.FromVisual(this);
            Point dipCenter = source != null && source.CompositionTarget != null
                ? source.CompositionTarget.TransformFromDevice.Transform(deviceCenter)
                : deviceCenter;
            Left = dipCenter.X - (ActualWidth / 2.0);
            Top = dipCenter.Y - (ActualHeight / 2.0);
        }

        internal static double MeasureFaceTextWidth(string value)
        {
            TextBlock probe = new TextBlock
            {
                Text = value,
                FontFamily = FaceFontFamily,
                FontSize = FaceFontSize,
                FontWeight = FontWeights.Normal,
                TextWrapping = TextWrapping.NoWrap
            };
            probe.Measure(new Size(Double.PositiveInfinity, Double.PositiveInfinity));
            return probe.DesiredSize.Width;
        }

        internal static double RequiredFaceWidth(string value)
        {
            return Math.Ceiling(MeasureFaceTextWidth(value) + (FaceHorizontalPadding * 2.0) + FaceWidthSafetyMargin);
        }

        private static Brush Brush(byte red, byte green, byte blue)
        {
            return new SolidColorBrush(Color.FromRgb(red, green, blue));
        }

        private void WriteStatus(TokenState state, CodexWindowInfo codexWindow, bool visible, string error)
        {
            try
            {
                string directory = Path.GetDirectoryName(options.StatusPath);
                if (!String.IsNullOrEmpty(directory)) Directory.CreateDirectory(directory);
                Native.Rect rect = new Native.Rect();
                bool hasRect = handle != IntPtr.Zero && Native.GetWindowRect(handle, out rect);
                long style = handle == IntPtr.Zero ? 0L : Native.GetWindowLongPtr(handle, Native.GwlExStyle).ToInt64();
                Dictionary<string, object> value = new Dictionary<string, object>();
                value["mode"] = "native-exe";
                value["process_id"] = Process.GetCurrentProcess().Id;
                value["overlay_handle"] = handle == IntPtr.Zero ? null : String.Format(CultureInfo.InvariantCulture, "0x{0:X}", handle.ToInt64());
                value["visible"] = visible;
                value["native_left"] = hasRect ? (object)rect.Left : null;
                value["native_top"] = hasRect ? (object)rect.Top : null;
                value["native_width"] = hasRect ? (object)(rect.Right - rect.Left) : null;
                value["native_height"] = hasRect ? (object)(rect.Bottom - rect.Top) : null;
                value["has_tool_window"] = (style & Native.WsExToolWindow) != 0;
                value["has_no_activate"] = (style & Native.WsExNoActivate) != 0;
                value["has_transparent_input"] = (style & Native.WsExTransparent) != 0;
                value["session_id"] = selectedPath == null ? null : RolloutReader.ReadMetadata(selectedPath).SessionId;
                value["used_tokens"] = state == null ? null : (object)state.UsedTokens;
                value["context_window"] = state == null ? null : (object)state.ContextWindow;
                value["remaining_tokens"] = state == null ? null : (object)state.RemainingTokens;
                value["percent_remaining"] = state == null ? null : (object)state.PercentRemaining;
                value["is_stale"] = state == null ? null : (object)state.IsStale;
                value["selection_ambiguous"] = selectionAmbiguous;
                value["prompt_background"] = lastPalette == null ? null : String.Format(CultureInfo.InvariantCulture, "#{0:X2}{1:X2}{2:X2}", lastPalette.BackgroundR, lastPalette.BackgroundG, lastPalette.BackgroundB);
                value["prompt_theme"] = lastPalette == null ? null : lastPalette.IsLight ? "light" : "dark";
                value["prompt_center"] = lastPalette == null ? null : (object)lastPalette.PromptCenter;
                value["sidebar_open"] = lastPalette == null ? null : (object)lastPalette.SidebarOpen;
                value["codex_foreground"] = codexWindow != null && codexWindow.IsForeground;
                value["codex_minimized"] = codexWindow == null ? null : (object)codexWindow.IsMinimized;
                value["error"] = error;
                value["updated_utc"] = DateTime.UtcNow.ToString("o", CultureInfo.InvariantCulture);
                File.WriteAllText(options.StatusPath, TokenEngine.NewSerializer().Serialize(value), new System.Text.UTF8Encoding(false));
            }
            catch { }
        }
    }
}
