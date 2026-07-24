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
                int idleIoCount = TestIdleIo(serializer);
                bool malformedFailed = false;
                try { TokenEngine.FromLines(new[] { "not-json" }, DateTime.UtcNow, 60, null); }
                catch (InvalidDataException) { malformedFailed = true; }
                AssertEqual(true, malformedFailed, "malformed input");

                TokenState smallerWindow = TokenEngine.FromLines(
                    new[] { "{\"timestamp\":\"2026-07-20T12:00:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"last_token_usage\":{\"total_tokens\":112000},\"model_context_window\":258400}}}" },
                    DateTime.Parse("2026-07-20T12:00:01Z", CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal),
                    60,
                    null);
                AssertEqual(258400L, smallerWindow.ContextWindow, "non-1M host window is reported as-is");

                bool unusableWindowFailed = false;
                try
                {
                    TokenEngine.FromLines(
                        new[] { "{\"timestamp\":\"2026-07-20T12:00:00Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"last_token_usage\":{\"total_tokens\":112000},\"model_context_window\":0}}}" },
                        DateTime.Parse("2026-07-20T12:00:01Z", CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal),
                        60,
                        null);
                }
                catch (InvalidDataException) { unusableWindowFailed = true; }
                AssertEqual(true, unusableWindowFailed, "unusable host window");

                Dictionary<string, object> result = new Dictionary<string, object>();
                result["passed"] = true;
                result["token_cases"] = tokenCount;
                result["selection_cases"] = selectionCount;
                result["layout_cases"] = layoutCount;
                result["face_width_cases"] = faceWidthCount;
                result["idle_io_cases"] = idleIoCount;
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
            AssertEqual("1M", TickerWindow.FormatWindow(1008000L), "1M face window");
            AssertEqual("258K", TickerWindow.FormatWindow(258400L), "smaller face window");
            string[] faces = { "Context: 117,015 / 1M", "Context: 1,008,000 / 1M", "Context: 112,000 / 258K" };
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

        private static int TestIdleIo(JavaScriptSerializer serializer)
        {
            AssertEqual(false, TickerPollPolicy.ShouldReadRollout(false, false), "background Codex skips rollout I/O");
            AssertEqual(false, TickerPollPolicy.ShouldReadRollout(true, true), "minimized Codex skips rollout I/O");
            AssertEqual(true, TickerPollPolicy.ShouldReadRollout(true, false), "foreground Codex permits rollout I/O");

            DateTime baseTime = DateTime.Parse("2026-07-20T12:00:00Z", CultureInfo.InvariantCulture, DateTimeStyles.AdjustToUniversal);
            StatusWriteGate writeGate = new StatusWriteGate(TimeSpan.FromSeconds(60));
            AssertEqual(true, writeGate.ShouldWrite("hidden", baseTime), "initial status write");
            for (int second = 1; second <= 10; second++)
            {
                AssertEqual(false, writeGate.ShouldWrite("hidden", baseTime.AddSeconds(second)), "unchanged idle status suppression");
            }
            AssertEqual(1, writeGate.ApprovedWriteCount, "idle status write bound");
            AssertEqual(true, writeGate.ShouldWrite("visible", baseTime.AddSeconds(11)), "meaningful status change");
            AssertEqual(false, writeGate.ShouldWrite("visible", baseTime.AddSeconds(70)), "heartbeat not yet due");
            AssertEqual(true, writeGate.ShouldWrite("visible", baseTime.AddSeconds(71)), "status heartbeat");

            string testRoot = Path.Combine(Path.GetTempPath(), "1mct-native-idle-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(testRoot);
            try
            {
                string sessionId = "idle-session";
                string path = Path.Combine(testRoot, "rollout-" + sessionId + ".jsonl");
                Dictionary<string, object> metadata = new Dictionary<string, object>();
                metadata["type"] = "session_meta";
                metadata["payload"] = new Dictionary<string, object>
                {
                    { "id", sessionId },
                    { "originator", "Codex Desktop" },
                    { "thread_source", "root" }
                };
                File.WriteAllText(
                    path,
                    serializer.Serialize(metadata) + "\n" + TokenEvent(serializer, baseTime, 112000L) + "\n",
                    new UTF8Encoding(false));
                File.SetLastWriteTimeUtc(path, baseTime);

                RolloutPollCache cache = new RolloutPollCache(TimeSpan.FromSeconds(30));
                TokenState initial = cache.Poll(testRoot, null, 60, baseTime.AddSeconds(1));
                AssertEqual(112000L, initial.UsedTokens, "initial cached state");
                for (int second = 2; second <= 10; second++)
                {
                    cache.Poll(testRoot, null, 60, baseTime.AddSeconds(second));
                }
                AssertEqual(1, cache.SelectionScanCount, "unchanged candidate scan bound");
                AssertEqual(1, cache.StateReadCount, "unchanged rollout read bound");

                File.AppendAllText(path, TokenEvent(serializer, baseTime.AddSeconds(11), 113000L) + "\n", new UTF8Encoding(false));
                File.SetLastWriteTimeUtc(path, baseTime.AddSeconds(11));
                TokenState changed = cache.Poll(testRoot, null, 60, baseTime.AddSeconds(11));
                AssertEqual(113000L, changed.UsedTokens, "changed rollout is reread");
                AssertEqual(1, cache.SelectionScanCount, "file change avoids recursive rescan");
                AssertEqual(2, cache.StateReadCount, "changed rollout read count");

                cache.Poll(testRoot, null, 60, baseTime.AddSeconds(31));
                AssertEqual(2, cache.SelectionScanCount, "periodic recovery scan");
                AssertEqual(2, cache.StateReadCount, "recovery scan reuses unchanged state");
            }
            finally
            {
                if (Directory.Exists(testRoot)) Directory.Delete(testRoot, true);
            }
            return 1;
        }

        private static string TokenEvent(JavaScriptSerializer serializer, DateTime timestampUtc, long usedTokens)
        {
            Dictionary<string, object> info = new Dictionary<string, object>();
            info["last_token_usage"] = new Dictionary<string, object> { { "total_tokens", usedTokens } };
            info["model_context_window"] = 1008000L;
            Dictionary<string, object> record = new Dictionary<string, object>();
            record["timestamp"] = timestampUtc.ToUniversalTime().ToString("o", CultureInfo.InvariantCulture);
            record["type"] = "event_msg";
            record["payload"] = new Dictionary<string, object> { { "type", "token_count" }, { "info", info } };
            return serializer.Serialize(record);
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
