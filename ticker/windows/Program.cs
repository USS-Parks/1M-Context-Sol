using System;
using System.IO;
using System.Reflection;
using System.Threading;
using System.Windows;

[assembly: AssemblyTitle("1M Context Ticker")]
[assembly: AssemblyDescription("Live active-context ticker for Codex Desktop")]
[assembly: AssemblyCompany("1M Context Ticker")]
[assembly: AssemblyProduct("1M Context Ticker")]
[assembly: AssemblyCopyright("Copyright 2026")]
[assembly: AssemblyVersion("0.1.0.0")]
[assembly: AssemblyFileVersion("0.1.0.0")]

namespace OneMContextTicker
{
    internal sealed class Options
    {
        public string SessionsRoot { get; set; }
        public string ThreadId { get; set; }
        public string StatusPath { get; set; }
        public int StaleAfterSeconds { get; set; }
        public string SelfTestFixture { get; set; }
        public string SelfTestOutput { get; set; }

        public static Options Parse(string[] args)
        {
            Options options = new Options
            {
                SessionsRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".codex", "sessions"),
                StatusPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "CodexContextOverlay", "state", "native-runtime-status.json"),
                StaleAfterSeconds = 300
            };
            for (int index = 0; index < args.Length; index++)
            {
                string name = args[index];
                if (name == "--self-test")
                {
                    options.SelfTestFixture = Required(args, ref index, name);
                    options.SelfTestOutput = Required(args, ref index, name);
                }
                else if (name == "--sessions-root") options.SessionsRoot = Required(args, ref index, name);
                else if (name == "--thread-id") options.ThreadId = Required(args, ref index, name);
                else if (name == "--status-path") options.StatusPath = Required(args, ref index, name);
                else if (name == "--stale-after-seconds") options.StaleAfterSeconds = Int32.Parse(Required(args, ref index, name));
                else throw new ArgumentException("Unknown option: " + name);
            }
            if (options.StaleAfterSeconds < 1) throw new ArgumentException("--stale-after-seconds must be positive.");
            return options;
        }

        private static string Required(string[] args, ref int index, string option)
        {
            index++;
            if (index >= args.Length || String.IsNullOrWhiteSpace(args[index])) throw new ArgumentException(option + " requires a value.");
            return args[index];
        }
    }

    internal static class Program
    {
        [STAThread]
        private static int Main(string[] args)
        {
            try
            {
                Options options = Options.Parse(args);
                if (!String.IsNullOrEmpty(options.SelfTestFixture)) return SelfTest.Run(options.SelfTestFixture, options.SelfTestOutput);

                bool created;
                using (Mutex mutex = new Mutex(true, "Local\\OneMContextTicker", out created))
                {
                    if (!created) return 0;
                    Application application = new Application { ShutdownMode = ShutdownMode.OnExplicitShutdown };
                    TickerWindow window = new TickerWindow(options);
                    int result = application.Run(window);
                    try { mutex.ReleaseMutex(); } catch (ApplicationException) { }
                    return result;
                }
            }
            catch (Exception error)
            {
                try
                {
                    string path = Path.Combine(Path.GetTempPath(), "1m-context-ticker-startup-error.txt");
                    File.WriteAllText(path, error.ToString());
                }
                catch { }
                return 1;
            }
        }
    }
}
