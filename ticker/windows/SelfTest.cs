using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Web.Script.Serialization;

namespace OneMContextTicker
{
    internal static class SelfTest
    {
        public static int Run(string fixturePath, string outputPath)
        {
            try
            {
                JavaScriptSerializer serializer = TokenEngine.NewSerializer();
                Dictionary<string, object> root = JsonValue.Object(serializer.DeserializeObject(File.ReadAllText(fixturePath)), "fixture");
                AssertEqual(1, JsonValue.Int32(root, "schema_version"), "schema version");
                AssertEqual((long)TokenEngine.BaselineTokens, JsonValue.Integer(root, "baseline_tokens"), "baseline");
                int tokenCount = TestTokens(serializer, root);
                int selectionCount = TestSelection(root);
                int layoutCount = TestLayout(root);
                int faceWidthCount = TestFaceWidth();
                bool malformedFailed = false;
                try { TokenEngine.FromLines(new[] { "not-json" }, DateTime.UtcNow, 60, null); }
                catch (InvalidDataException) { malformedFailed = true; }
                AssertEqual(true, malformedFailed, "malformed input");

                bool wrongWindowFailed = false;
                try
                {
                    TokenEngine.FromLines(
                        new[] { "{\"timestamp\":\"2026-07-20T12:00:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"last_token_usage\":{\"total_tokens\":112000},\"model_context_window\":258400}}}" },
                        DateTime.Parse("2026-07-20T12:00:01Z", CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal),
                        60,
                        null);
                }
                catch (InvalidDataException) { wrongWindowFailed = true; }
                AssertEqual(true, wrongWindowFailed, "non-1M host window");

                Dictionary<string, object> result = new Dictionary<string, object>();
                result["passed"] = true;
                result["token_cases"] = tokenCount;
                result["selection_cases"] = selectionCount;
                result["layout_cases"] = layoutCount;
                result["face_width_cases"] = faceWidthCount;
                result["window_guard_cases"] = 1;
                result["executable"] = System.Reflection.Assembly.GetExecutingAssembly().Location;
                File.WriteAllText(outputPath, serializer.Serialize(result), new UTF8Encoding(false));
                return 0;
            }
            catch (Exception error)
            {
                try
                {
                    Dictionary<string, object> failed = new Dictionary<string, object>();
                    failed["passed"] = false;
                    failed["error"] = error.ToString();
                    File.WriteAllText(outputPath, TokenEngine.NewSerializer().Serialize(failed), new UTF8Encoding(false));
                }
                catch { }
                return 1;
            }
        }

        private static int TestTokens(JavaScriptSerializer serializer, Dictionary<string, object> root)
        {
            object[] cases = JsonValue.Required(root, "token_cases") as object[];
            if (cases == null) throw new InvalidDataException("token_cases must be an array.");
            foreach (object value in cases)
            {
                Dictionary<string, object> item = JsonValue.Object(value, "token case");
                Dictionary<string, object> info = new Dictionary<string, object>();
                info["total_token_usage"] = new Dictionary<string, object> { { "total_tokens", JsonValue.Integer(item, "cumulative_total_tokens") } };
                info["last_token_usage"] = new Dictionary<string, object> { { "total_tokens", JsonValue.Integer(item, "active_total_tokens") } };
                info["model_context_window"] = JsonValue.Integer(item, "context_window");
                Dictionary<string, object> record = new Dictionary<string, object>();
                record["timestamp"] = JsonValue.Text(item, "timestamp");
                record["type"] = "event_msg";
                record["payload"] = new Dictionary<string, object> { { "type", "token_count" }, { "info", info } };
                object previousValue;
                long? previous = item.TryGetValue("previous_used_tokens", out previousValue) && previousValue != null
                    ? (long?)Convert.ToInt64(previousValue, CultureInfo.InvariantCulture)
                    : null;
                TokenState state = TokenEngine.FromLines(
                    new[] { serializer.Serialize(record), "malformed-json" },
                    DateTime.Parse(JsonValue.Text(item, "now"), CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal | DateTimeStyles.AssumeUniversal),
                    JsonValue.Int32(item, "stale_after_seconds"),
                    previous);
                Dictionary<string, object> expected = JsonValue.Object(JsonValue.Required(item, "expected"), "expected");
                string id = JsonValue.Text(item, "id");
                AssertEqual(JsonValue.Integer(expected, "used_tokens"), state.UsedTokens, id + " used");
                AssertEqual(JsonValue.Integer(expected, "effective_window"), state.EffectiveWindow, id + " effective");
                AssertEqual(JsonValue.Integer(expected, "remaining_tokens"), state.RemainingTokens, id + " remaining");
                AssertEqual(JsonValue.Int32(expected, "percent_remaining"), state.PercentRemaining, id + " percent");
                AssertEqual(JsonValue.Boolean(expected, "is_stale"), state.IsStale, id + " stale");
                AssertEqual(JsonValue.Boolean(expected, "was_compacted"), state.WasCompacted, id + " compacted");
            }
            return cases.Length;
        }

        private static int TestSelection(Dictionary<string, object> root)
        {
            object[] cases = JsonValue.Required(root, "selection_cases") as object[];
            if (cases == null) throw new InvalidDataException("selection_cases must be an array.");
            string testRoot = Path.Combine(Path.GetTempPath(), "1mct-native-selection-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(testRoot);
            try
            {
                DateTime baseTime = DateTime.Parse("2026-07-20T12:00:00Z", CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal);
                foreach (object value in cases)
                {
                    Dictionary<string, object> item = JsonValue.Object(value, "selection case");
                    string id = JsonValue.Text(item, "id");
                    string caseRoot = Path.Combine(testRoot, id);
                    Directory.CreateDirectory(caseRoot);
                    object[] candidates = JsonValue.Required(item, "candidates") as object[];
                    foreach (object candidateValue in candidates)
                    {
                        Dictionary<string, object> candidate = JsonValue.Object(candidateValue, "candidate");
                        string sessionId = JsonValue.Text(candidate, "session_id");
                        Dictionary<string, object> payload = new Dictionary<string, object>();
                        payload["id"] = sessionId;
                        payload["originator"] = "Codex Desktop";
                        payload["thread_source"] = JsonValue.Text(candidate, "thread_source");
                        Dictionary<string, object> meta = new Dictionary<string, object>();
                        meta["type"] = "session_meta";
                        meta["payload"] = payload;
                        string path = Path.Combine(caseRoot, "rollout-" + sessionId + ".jsonl");
                        File.WriteAllText(path, TokenEngine.NewSerializer().Serialize(meta), new UTF8Encoding(false));
                        File.SetLastWriteTimeUtc(path, baseTime.AddSeconds(JsonValue.Int32(candidate, "last_write_offset_seconds")));
                    }
                    object explicitValue;
                    string explicitId = item.TryGetValue("explicit_thread_id", out explicitValue) && explicitValue != null
                        ? Convert.ToString(explicitValue, CultureInfo.InvariantCulture)
                        : null;
                    RolloutSelection selection = RolloutSelector.Select(caseRoot, explicitId, 15);
                    AssertEqual(JsonValue.Text(item, "expected_session_id"), selection.SessionId, id + " selected");
                    AssertEqual(JsonValue.Boolean(item, "expected_ambiguous"), selection.Ambiguous, id + " ambiguous");
                }
            }
            finally
            {
                if (Directory.Exists(testRoot)) Directory.Delete(testRoot, true);
            }
            return cases.Length;
        }

        private static int TestLayout(Dictionary<string, object> root)
        {
            object[] cases = JsonValue.Required(root, "layout_cases") as object[];
            if (cases == null) throw new InvalidDataException("layout_cases must be an array.");
            foreach (object value in cases)
            {
                Dictionary<string, object> item = JsonValue.Object(value, "layout case");
                int actual = Native.ComputeComposerCenter(
                    JsonValue.Int32(item, "window_left"),
                    JsonValue.Int32(item, "window_right"),
                    JsonValue.Boolean(item, "sidebar_open"));
                AssertEqual(JsonValue.Int32(item, "expected_center"), actual, JsonValue.Text(item, "id") + " center");
            }
            return cases.Length;
        }

        private static int TestFaceWidth()
        {
            string[] faces = { "Context: 117,015 / 1M", "Context: 1,008,000 / 1M" };
            foreach (string face in faces)
            {
                double textWidth = TickerWindow.MeasureFaceTextWidth(face);
                double requiredWidth = TickerWindow.RequiredFaceWidth(face);
                if (requiredWidth < textWidth + 22.0)
                {
                    throw new InvalidDataException("Ticker face width does not preserve padding and safety margin: " + face);
                }
            }
            return faces.Length;
        }

        private static void AssertEqual(object expected, object actual, string label)
        {
            if (!Object.Equals(expected, actual))
            {
                throw new InvalidDataException(String.Format(CultureInfo.InvariantCulture, "{0} expected '{1}' but got '{2}'.", label, expected, actual));
            }
        }
    }
}
