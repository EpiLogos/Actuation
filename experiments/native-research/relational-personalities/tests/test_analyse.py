"""Synthetic unit cases for derived arithmetic, not native evidence fixtures."""
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("analysis", Path(__file__).parents[1] / "analyse.py")
analysis = importlib.util.module_from_spec(spec)
spec.loader.exec_module(analysis)


def example():
    return {"schema": "actuation.acquaintance-evidence-projection/v1", "episode_ref": "synthetic-test:episode",
            "participant_refs": ["synthetic-test:a", "synthetic-test:b", "synthetic-test:c", "synthetic-test:d"],
            "native_basis_refs": ["synthetic-test:unverified-basis"], "messages": [], "observations": [], "usage": {"tokens": None}}


class AnalysisTests(unittest.TestCase):
    def test_missing_opportunity_is_null(self):
        result = analysis.summarise(example())
        self.assertIsNone(result["metrics"]["correction_uptake"]["rate"])
        self.assertIsNone(result["usage"]["tokens"])
        self.assertFalse(result["native_integrity_assessed_by_this_tool"])

    def test_exact_ratio(self):
        doc = example()
        doc["observations"] = [{"metric": "contingent_uptake", "opportunity_ref": f"synthetic-test:{i}",
                                "eligible": True, "satisfied": passed, "annotator_ref": "synthetic-test:rater",
                                "evidence_refs": [f"synthetic-test:source-{i}"]} for i, passed in enumerate([True, False, True])]
        result = analysis.summarise(doc)["metrics"]["contingent_uptake"]
        self.assertEqual((result["numerator"], result["denominator"]), (2, 3))
        self.assertAlmostEqual(result["rate"], 2 / 3)

    def test_duplicate_message_rejected(self):
        doc = example()
        m = {"native_ref": "synthetic-test:m", "sender_ref": "synthetic-test:a", "segment": "first-meeting"}
        doc["messages"] = [m, m]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_four_distinct_required(self):
        doc = example(); doc["participant_refs"][3] = doc["participant_refs"][0]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_unknown_sender_rejected(self):
        doc = example(); doc["messages"] = [{"native_ref": "m", "sender_ref": "other", "segment": "first-meeting"}]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_reciprocal_dyad_counts_once(self):
        doc = example(); doc["messages"] = [
            {"native_ref": "m1", "sender_ref": "synthetic-test:a", "segment": "first-meeting"},
            {"native_ref": "m2", "sender_ref": "synthetic-test:b", "segment": "first-meeting", "reply_to": "m1"},
            {"native_ref": "m3", "sender_ref": "synthetic-test:a", "segment": "return-meeting", "reply_to": "m2"}]
        self.assertEqual(analysis.summarise(doc)["reciprocal_dyads"], {"count": 1, "possible": 6})

    def test_dangling_reply_rejected(self):
        doc = example(); doc["messages"] = [{"native_ref": "m1", "sender_ref": "synthetic-test:a", "segment": "first-meeting", "reply_to": "missing"}]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_uncited_annotation_rejected(self):
        doc = example(); doc["observations"] = [{"metric": "attribution", "opportunity_ref": "x", "eligible": True, "satisfied": True, "annotator_ref": "r", "evidence_refs": []}]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_ineligible_cannot_be_success(self):
        doc = example(); doc["observations"] = [{"metric": "attribution", "opportunity_ref": "x", "eligible": False, "satisfied": True, "annotator_ref": "r", "evidence_refs": ["e"]}]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_duplicate_observation_rejected(self):
        doc = example(); o = {"metric": "attribution", "opportunity_ref": "x", "eligible": True, "satisfied": True, "annotator_ref": "r", "evidence_refs": ["e"]}
        doc["observations"] = [o, o]
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_nonfinite_usage_rejected(self):
        doc = example(); doc["usage"] = {"tokens": float("nan")}
        with self.assertRaises(ValueError): analysis.summarise(doc)

    def test_reply_to_opening_event_is_not_a_fifth_peer(self):
        doc = example(); doc["nonparticipant_event_refs"] = ["synthetic-test:opening"]
        doc["messages"] = [{"native_ref": "m1", "sender_ref": "synthetic-test:a", "segment": "first-meeting", "reply_to": "synthetic-test:opening"}]
        result = analysis.summarise(doc)
        self.assertEqual(result["reciprocal_dyads"]["count"], 0)
        self.assertEqual(result["public_acts_by_participant"]["synthetic-test:a"], 1)

    def test_missing_native_basis_rejected(self):
        doc = example(); doc["native_basis_refs"] = []
        with self.assertRaises(ValueError): analysis.summarise(doc)


if __name__ == "__main__":
    unittest.main()
