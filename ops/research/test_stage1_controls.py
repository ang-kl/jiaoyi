import unittest

import stage1_controls as controls


class FrozenControlAccountingTests(unittest.TestCase):
    def test_control_denominators_preserve_veto_and_censor(self):
        rows = [
            {"controls": {"always_long": {"status": "cost_ge_r_session_veto"}}},
            {"controls": {"always_long": {"status": "simulated_entry", "control": {
                "censored": True, "net_by_bps": {}}}}},
            {"controls": {"always_long": {"status": "simulated_entry", "control": {
                "censored": False, "net_by_bps": {"3.74": 0.25, "24": -0.1}}}}},
            {"controls": {"always_long": {"status": "simulated_entry", "control": {
                "censored": False, "net_by_bps": {"3.74": -0.5, "24": -0.9}}}}},
        ]
        actual = controls.control_score(rows, "always_long", 3.74)
        self.assertEqual((actual["episodes"], actual["entries"], actual["closed"], actual["censored"]),
                         (4, 3, 2, 1))
        self.assertEqual((actual["wins"], actual["mean_net_r"], actual["profit_factor"]), (1, -0.125, 0.5))
        stress = controls.control_score(rows, "always_long", 24.0)
        self.assertEqual((stress["wins"], stress["mean_net_r"]), (0, -0.5))

    def test_absent_cost_or_perfect_sample_does_not_invent_profit_factor(self):
        row = {"controls": {"always_long": {"status": "simulated_entry", "control": {
            "censored": False, "net_by_bps": {"3.74": 0.4}}}}}
        self.assertIsNone(controls.control_score([row], "always_long", 3.74)["profit_factor"])
        with self.assertRaisesRegex(ValueError, "Missing requested control cost"):
            controls.control_score([row], "always_long", 24)


if __name__ == "__main__":
    unittest.main()
