"""Pydantic AI's native Python ABI. The Rust parent owns context construction,
model-output interpretation, bounds, chronology and experimental comparison."""
import importlib.metadata
import json
import sys


def observed(usage, name):
    value = getattr(usage, name, None)
    return value if isinstance(value, int) and not isinstance(value, bool) and value >= 0 else None


class PydanticBody:
    def __init__(self):
        self.config = None
        self.model = None
        self.calls = []

    def preflight(self, config):
        if self.config is not None:
            raise ValueError('Pydantic body cannot be rebound')
        if config.get('provider') != 'deepseek' or not config.get('model'):
            raise ValueError('Explicit DeepSeek model required')
        from pydantic_ai import Agent
        from pydantic_ai.models.openai import OpenAIChatModel
        from pydantic_ai.providers.deepseek import DeepSeekProvider
        self.model = OpenAIChatModel(config['model'], provider=DeepSeekProvider())
        Agent(self.model)
        self.config = {'provider': config['provider'], 'model': config['model']}
        return {'ready': True, **self.config, 'framework': 'pydantic-ai',
                'package_version': importlib.metadata.version('pydantic-ai-slim'),
                'provider_request_executed': False}

    def complete(self, request):
        if self.config is None:
            raise ValueError('Pydantic body requires preflight')
        from pydantic_ai import Agent
        ordinal = len(self.calls)
        try:
            result = Agent(self.model, system_prompt=request['system']).run_sync(
                request['prompt'], model_settings={'temperature': request.get('temperature', 0)})
            usage = result.usage() if callable(getattr(result, 'usage', None)) else getattr(result, 'usage', None)
            output = result.output
            messages = result.all_messages()
            model_name = getattr(messages[-1], 'model_name', None) if messages else None
            raw = {'framework': 'pydantic-ai', 'model': model_name,
                   'message_count': len(messages), 'provider': self.config['provider']}
            self.calls.append({'ordinal': ordinal, 'status': 'returned', 'raw': raw})
            return {'output': output if isinstance(output, str) else json.dumps(output, default=str),
                    'usage': {name: observed(usage, name) for name in (
                        'input_tokens', 'output_tokens', 'total_tokens')} if usage else None, 'raw': raw}
        except Exception as exc:
            self.calls.append({'ordinal': ordinal, 'status': 'failed', 'error_kind': type(exc).__name__})
            raise RuntimeError('Pydantic SDK completion failed; failure retained in native evidence') from None

    def finalize(self, _inspection=None):
        return {'kind': 'pydantic-native-sdk-evidence', 'framework': 'pydantic-ai',
                'model_calls': self.calls, 'session_evidence': None, 'provider_evidence': 'not-assessed'}


def main():
    body = PydanticBody()
    while True:
        line = sys.stdin.buffer.readline(16 * 1024 * 1024 + 1)
        if not line:
            return
        if len(line) > 16 * 1024 * 1024 or not line.endswith(b'\n'):
            raise ValueError('SDK frame exceeds bound or is incomplete')
        v = json.loads(line)
        data, error = None, None
        try:
            if v.get('type') == 'preflight':
                data = body.preflight(v['configuration'])
            elif v.get('type') == 'complete':
                data = body.complete(v['request'])
            elif v.get('type') == 'finalize':
                data = body.finalize(v.get('inspection'))
            else:
                raise ValueError('Unsupported Pydantic adapter command')
        except Exception as exc:
            # Construction errors contain no provider response/body or credential.
            error = str(exc) if isinstance(exc, (ValueError, RuntimeError)) else type(exc).__name__
        out = json.dumps({'type': 'response', 'command': v.get('type'), 'id': v.get('id'),
                          'success': error is None, 'data': data, 'error': error})
        if len(out.encode()) > 16 * 1024 * 1024:
            raise ValueError('SDK response exceeds bound')
        print(out, flush=True)
        if v.get('type') == 'finalize':
            return


if __name__ == '__main__':
    try:
        main()
    except Exception:
        sys.stderr.write('Pydantic SDK transport failed\n')
        raise SystemExit(1)
