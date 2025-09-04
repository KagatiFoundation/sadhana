from core.pipeline import EnginePipeline
import asyncio

async def main():
    pipeline = EnginePipeline()
    await pipeline.process_batch(
        [
            'https://example.com',
            'https://hello.com',
        ]
    )

try:
    asyncio.run(main())
except KeyboardInterrupt:
    print("Shutdown...")