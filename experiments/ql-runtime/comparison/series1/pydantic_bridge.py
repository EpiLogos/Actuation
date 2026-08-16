import json
import sys


def emit(value):
    sys.stdout.write(json.dumps(value, default=str))
    sys.stdout.flush()


def usage_dict(result):
    try:
        usage = result.usage() if callable(getattr(result, 'usage', None)) else getattr(result, 'usage', None)
    except Exception:
        usage = None
    if usage is None:
        return None
    return {
        'input_tokens': int(getattr(usage, 'input_tokens', 0) or 0),
        'output_tokens': int(getattr(usage, 'output_tokens', 0) or 0),
        'total_tokens': int(getattr(usage, 'total_tokens', 0) or 0),
    }


def build_model(model_name):
    from pydantic_ai.models.openai import OpenAIChatModel
    from pydantic_ai.providers.deepseek import DeepSeekProvider

    # DeepSeekProvider discovers DEEPSEEK_API_KEY itself. Keeping this explicit
    # avoids treating DeepSeek as a generic OpenAI provider and pins the real
    # Pydantic AI provider path being evaluated.
    return OpenAIChatModel(model_name, provider=DeepSeekProvider())


def main():
    payload = json.load(sys.stdin)
    operation = payload.get('operation')
    model_name = payload.get('model')
    if not model_name:
        emit({'ready': False, 'error': 'A concrete DeepSeek model id is required.'})
        return 2

    try:
        from pydantic_ai import Agent
    except Exception as exc:
        emit({'ready': False, 'error': f'Real pydantic_ai import failed: {exc}'})
        return 3

    if operation == 'preflight':
        try:
            model = build_model(model_name)
            Agent(model)
            emit({'ready': True, 'model': model_name, 'provider': 'deepseek', 'framework': 'pydantic-ai'})
            return 0
        except Exception as exc:
            emit({'ready': False, 'error': f'Pydantic DeepSeek model construction failed: {exc}'})
            return 4

    if operation != 'complete':
        emit({'ready': False, 'error': f'Unknown operation {operation!r}'})
        return 5

    try:
        model = build_model(model_name)
        agent = Agent(model, system_prompt=payload.get('system') or '')
        result = agent.run_sync(payload.get('prompt') or '')
        output = result.output if hasattr(result, 'output') else getattr(result, 'data', '')
        returned_model_name = None
        try:
            messages = result.all_messages()
            if messages:
                returned_model_name = getattr(messages[-1], 'model_name', None)
        except Exception:
            pass
        emit({
            'ready': True,
            'output': output if isinstance(output, str) else json.dumps(output, default=str),
            'usage': usage_dict(result),
            'model_name': returned_model_name or model_name,
            'provider': 'deepseek',
        })
        return 0
    except Exception as exc:
        emit({'ready': False, 'error': f'Pydantic live DeepSeek completion failed: {exc}'})
        return 6


if __name__ == '__main__':
    raise SystemExit(main())
