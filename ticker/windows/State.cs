using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Web.Script.Serialization;

namespace OneMContextTicker
{
    internal sealed class TokenState
    {
        public long UsedTokens { get; set; }
        public long ContextWindow { get; set; }
        public long EffectiveWindow { get; set; }
        public long RemainingTokens { get; set; }
        public int PercentRemaining { get; set; }
        public string EventTimestampUtc { get; set; }
        public int AgeSeconds { get; set; }
        public bool IsStale { get; set; }
        public bool WasCompacted { get; set; }
    }

    internal sealed class RolloutMetadata
    {
        public string Path { get; set; }
        public string SessionId { get; set; }
        public string ThreadSource { get; set; }
        public DateTime LastWriteUtc { get; set; }
    }

    internal sealed class RolloutSelection
    {
        public string Path { get; set; }
        public string SessionId { get; set; }
        public string ShortId { get; set; }
        public bool Ambiguous { get; set; }
    }

    internal static class JsonValue
    {
        public static Dictionary<string, object> Object(object value, string label)
        {
            Dictionary<string, object> result = value as Dictionary<string, object>;
            if (result == null) throw new InvalidDataException(label + " must be an object.");
            return result;
        }

        public static object Required(Dictionary<string, object> value, string key)
        {
            object result;
            if (!value.TryGetValue(key, out result)) throw new InvalidDataException("Missing JSON field: " + key);
            return result;
        }

        public static string Text(Dictionary<string, object> value, string key)
        {
            return Convert.ToString(Required(value, key), CultureInfo.InvariantCulture);
        }

        public static long Integer(Dictionary<string, object> value, string key)
        {
            return Convert.ToInt64(Required(value, key), CultureInfo.InvariantCulture);
        }

        public static int Int32(Dictionary<string, object> value, string key)
        {
            return Convert.ToInt32(Required(value, key), CultureInfo.InvariantCulture);
        }

        public static bool Boolean(Dictionary<string, object> value, string key)
        {
            return Convert.ToBoolean(Required(value, key), CultureInfo.InvariantCulture);
        }
    }

    internal static class TokenEngine
    {
        public const long BaselineTokens = 12000L;
        public const long RequiredHostWindow = 1008000L;

        public static TokenState FromLines(
            IEnumerable<string> lines,
            DateTime nowUtc,
            int staleAfterSeconds,
            long? previousUsedTokens)
        {
            string[] values = lines.Where(delegate(string line) { return !String.IsNullOrWhiteSpace(line); }).ToArray();
            JavaScriptSerializer serializer = NewSerializer();
            Dictionary<string, object> record = null;

            for (int index = values.Length - 1; index >= 0; index--)
            {
                try
                {
                    Dictionary<string, object> candidate = serializer.DeserializeObject(values[index]) as Dictionary<string, object>;
                    if (candidate == null || JsonValue.Text(candidate, "type") != "event_msg") continue;
                    Dictionary<string, object> payload = JsonValue.Object(JsonValue.Required(candidate, "payload"), "payload");
                    if (JsonValue.Text(payload, "type") == "token_count")
                    {
                        record = candidate;
                        break;
                    }
                }
                catch (Exception error)
                {
                    if (error is OutOfMemoryException) throw;
                }
            }

            if (record == null) throw new InvalidDataException("No valid token_count event was found.");
            Dictionary<string, object> tokenPayload = JsonValue.Object(JsonValue.Required(record, "payload"), "payload");
            Dictionary<string, object> info = JsonValue.Object(JsonValue.Required(tokenPayload, "info"), "info");
            Dictionary<string, object> last = JsonValue.Object(JsonValue.Required(info, "last_token_usage"), "last_token_usage");
            long used = JsonValue.Integer(last, "total_tokens");
            long contextWindow = JsonValue.Integer(info, "model_context_window");
            if (contextWindow != RequiredHostWindow)
            {
                throw new InvalidDataException(String.Format(
                    CultureInfo.InvariantCulture,
                    "Host context window {0} does not match required 1M budget {1}.",
                    contextWindow,
                    RequiredHostWindow));
            }
            if (used < 0) throw new InvalidDataException("Invalid active token count.");

            long effective = contextWindow - BaselineTokens;
            long adjustedUsed = Math.Max(used - BaselineTokens, 0L);
            long remaining = Math.Max(effective - adjustedUsed, 0L);
            int percent = (int)Math.Floor(((remaining / (double)effective) * 100.0) + 0.5);
            DateTime timestamp = DateTime.Parse(
                JsonValue.Text(record, "timestamp"),
                CultureInfo.InvariantCulture,
                DateTimeStyles.AdjustToUniversal | DateTimeStyles.AssumeUniversal);
            int age = Math.Max(0, (int)(nowUtc.ToUniversalTime() - timestamp.ToUniversalTime()).TotalSeconds);

            return new TokenState
            {
                UsedTokens = used,
                ContextWindow = contextWindow,
                EffectiveWindow = effective,
                RemainingTokens = remaining,
                PercentRemaining = percent,
                EventTimestampUtc = timestamp.ToUniversalTime().ToString("o", CultureInfo.InvariantCulture),
                AgeSeconds = age,
                IsStale = age > staleAfterSeconds,
                WasCompacted = previousUsedTokens.HasValue && used < previousUsedTokens.Value
            };
        }

        public static JavaScriptSerializer NewSerializer()
        {
            JavaScriptSerializer serializer = new JavaScriptSerializer();
            serializer.MaxJsonLength = 32 * 1024 * 1024;
            serializer.RecursionLimit = 128;
            return serializer;
        }
    }

    internal static class RolloutReader
    {
        private static readonly int[] TailSizes = { 262144, 1048576, 4194304, 16777216 };

        public static TokenState ReadState(string path, int staleAfterSeconds, long? previousUsedTokens)
        {
            Exception lastError = null;
            foreach (int size in TailSizes)
            {
                try
                {
                    return TokenEngine.FromLines(ReadTailLines(path, size), DateTime.UtcNow, staleAfterSeconds, previousUsedTokens);
                }
                catch (InvalidDataException error)
                {
                    lastError = error;
                }
            }
            throw lastError ?? new InvalidDataException("No token state was available.");
        }

        public static IList<string> ReadTailLines(string path, int maximumBytes)
        {
            using (FileStream stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete))
            {
                int count = (int)Math.Min((long)maximumBytes, stream.Length);
                long start = stream.Length - count;
                stream.Seek(start, SeekOrigin.Begin);
                byte[] buffer = new byte[count];
                int read = 0;
                while (read < count)
                {
                    int chunk = stream.Read(buffer, read, count - read);
                    if (chunk == 0) break;
                    read += chunk;
                }
                string text = Encoding.UTF8.GetString(buffer, 0, read);
                string[] lines = text.Split(new[] { "\r\n", "\n" }, StringSplitOptions.RemoveEmptyEntries);
                int skip = start > 0 && lines.Length > 0 ? 1 : 0;
                return lines.Skip(skip).ToArray();
            }
        }

        public static RolloutMetadata ReadMetadata(string path)
        {
            try
            {
                string firstLine;
                using (FileStream stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete))
                using (StreamReader reader = new StreamReader(stream, Encoding.UTF8, true, 4096))
                {
                    firstLine = reader.ReadLine();
                }
                if (String.IsNullOrWhiteSpace(firstLine)) return null;
                Dictionary<string, object> record = TokenEngine.NewSerializer().DeserializeObject(firstLine) as Dictionary<string, object>;
                if (record == null || JsonValue.Text(record, "type") != "session_meta") return null;
                Dictionary<string, object> payload = JsonValue.Object(JsonValue.Required(record, "payload"), "payload");
                if (JsonValue.Text(payload, "originator") != "Codex Desktop") return null;
                return new RolloutMetadata
                {
                    Path = path,
                    SessionId = JsonValue.Text(payload, "id"),
                    ThreadSource = payload.ContainsKey("thread_source") ? Convert.ToString(payload["thread_source"], CultureInfo.InvariantCulture) : String.Empty,
                    LastWriteUtc = File.GetLastWriteTimeUtc(path)
                };
            }
            catch (Exception error)
            {
                if (error is OutOfMemoryException) throw;
                return null;
            }
        }
    }

    internal static class RolloutSelector
    {
        public static RolloutSelection Select(string sessionsRoot, string explicitThreadId, int ambiguousWithinSeconds)
        {
            if (!Directory.Exists(sessionsRoot)) throw new DirectoryNotFoundException("Codex sessions root does not exist: " + sessionsRoot);
            List<FileInfo> files = Directory.GetFiles(sessionsRoot, "rollout-*.jsonl", SearchOption.AllDirectories)
                .Select(delegate(string path) { return new FileInfo(path); })
                .OrderByDescending(delegate(FileInfo file) { return file.LastWriteTimeUtc; })
                .ToList();
            List<RolloutMetadata> candidates = new List<RolloutMetadata>();

            foreach (FileInfo file in files)
            {
                if (!String.IsNullOrEmpty(explicitThreadId) && file.Name.IndexOf(explicitThreadId, StringComparison.OrdinalIgnoreCase) < 0) continue;
                RolloutMetadata metadata = RolloutReader.ReadMetadata(file.FullName);
                if (metadata == null) continue;
                if (String.IsNullOrEmpty(explicitThreadId) && metadata.ThreadSource == "subagent") continue;
                if (!String.IsNullOrEmpty(explicitThreadId) && metadata.SessionId != explicitThreadId) continue;
                candidates.Add(metadata);
                if (!String.IsNullOrEmpty(explicitThreadId) || candidates.Count >= 2) break;
            }

            if (candidates.Count == 0) throw new InvalidDataException("No matching root Codex Desktop rollout was found.");
            RolloutMetadata selected = candidates[0];
            bool ambiguous = false;
            if (String.IsNullOrEmpty(explicitThreadId) && candidates.Count > 1)
            {
                ambiguous = Math.Abs((selected.LastWriteUtc - candidates[1].LastWriteUtc).TotalSeconds) <= ambiguousWithinSeconds;
            }
            return new RolloutSelection
            {
                Path = selected.Path,
                SessionId = selected.SessionId,
                ShortId = selected.SessionId.Length <= 8 ? selected.SessionId : selected.SessionId.Substring(selected.SessionId.Length - 8),
                Ambiguous = ambiguous
            };
        }
    }
}
