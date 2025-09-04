from core.pipeline import EnginePipeline
import asyncio

async def main():
    pipeline = EnginePipeline()
    '''
    await pipeline.process_batch(
        [
            'https://example.com',
            'https://hello.com',
        ]
    )
    '''
    print(pipeline.db_handle.lookup_word_doc_mapping('lugar'))

try:
    asyncio.run(main())
except KeyboardInterrupt:
    print("Shutdown...")