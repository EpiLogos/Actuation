"""Exercise the actual pinned Pydantic model/Agent ABI, not a provider turn."""
import importlib.util
import os
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('native_pydantic_body', Path(__file__).parents[1] / 'pydantic.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class NativePydanticTests(unittest.TestCase):
    def test_real_agent_with_controlled_model_and_ordered_failure(self):
        from pydantic_ai.models.test import TestModel
        from pydantic_ai.models.function import FunctionModel
        old = os.environ.get('DEEPSEEK_API_KEY')
        os.environ['DEEPSEEK_API_KEY'] = 'deterministic-not-sent'
        try:
            body = module.PydanticBody()
            preflight = body.preflight({'provider': 'deepseek', 'model': 'deepseek-v4-flash'})
            self.assertFalse(preflight['provider_request_executed'])
            seen = []

            def fail(messages, info):
                seen.extend(messages)
                raise RuntimeError('deliberate fixture failure')

            body.model = FunctionModel(fail)
            with self.assertRaises(RuntimeError):
                body.complete({'system': 'authored system', 'prompt': 'first'})
            body.model = TestModel(custom_output_text='{"content":"fixture","capabilityCalls":[]}')
            result = body.complete({'system': 'authored system', 'prompt': 'second'})
            self.assertEqual(result['output'], '{"content":"fixture","capabilityCalls":[]}')
            self.assertGreater(result['raw']['message_count'], 0)
            self.assertTrue(seen)
            self.assertEqual([(c['ordinal'], c['status']) for c in body.finalize()['model_calls']],
                             [(0, 'failed'), (1, 'returned')])
            with self.assertRaises(ValueError):
                body.preflight({'provider': 'deepseek', 'model': 'other'})
            with self.assertRaises(ValueError):
                module.PydanticBody().preflight({'provider': 'other', 'model': 'x'})
        finally:
            if old is None:
                os.environ.pop('DEEPSEEK_API_KEY', None)
            else:
                os.environ['DEEPSEEK_API_KEY'] = old


if __name__ == '__main__':
    unittest.main()
