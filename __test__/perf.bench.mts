import { getChunk as getChunkJs, split as splitJs } from "@nearform/llm-chunk";
import { bench, describe } from "vitest";
import { ChunkStrategy, getChunk as getChunkRust, split as splitRust } from "..";

describe("split", () => {
  describe("simple string", () => {
    const input = ["abcdefghij"];
    bench("rust", () => {
      splitRust(input, { chunkSize: 3 });
    });

    bench("js", () => {
      splitJs(input, { chunkSize: 3 });
    });
  });

  describe("string array with correct size", () => {
    const input = ["abcde", "fghij"];
    bench("rust", () => {
      splitRust(input, { chunkSize: 2 });
    });

    bench("js", () => {
      splitJs(input, { chunkSize: 2 });
    });
  });

  describe("smaller than chunk size", () => {
    const input = ["abc"];
    bench("rust", () => {
      splitRust(input, { chunkSize: 10 });
    });

    bench("js", () => {
      splitJs(input, { chunkSize: 10 });
    });
  });

  describe("with overlap", () => {
    const input = ["abcdefghij"];
    bench("rust", () => {
      splitRust(input, { chunkSize: 4, chunkOverlap: 2 });
    });

    bench("js", () => {
      splitJs(input, { chunkSize: 4, chunkOverlap: 2 });
    });
  });

  describe("custom lengthFunction", () => {
    const input = ["abcdeiouxyz"];
    const options = {
      chunkSize: 2,
      lengthFunction: (t) => (t.match(/[aeiou]/g) || []).length,
    };

    bench("rust", () => {
      splitRust(input, options);
    });

    bench("js", () => {
      splitJs(input, options);
    });
  });

  describe("paragraph strategy", () => {
    const input = ["A", "B", "C"];

    bench("rust", () => {
      splitRust(input, {
        chunkSize: 10,
        chunkStrategy: ChunkStrategy.Paragraph,
      });
    });

    bench("js", () => {
      splitJs(input, { chunkSize: 10, chunkStrategy: "paragraph" });
    });
  });
});

describe("getChunk", () => {
    describe('full string', () => {
        const input = ['abcdefgh']

        bench('rust', () => {
            getChunkRust(input);
        })

        bench('js', () => {
            getChunkJs(input);
        })
    })

    describe('substring for start only', () => {
        const input = ['abcdefgh']

        bench('rust', () => {
            getChunkRust(input, 2);
        })

        bench('js', () => {
            getChunkJs(input, 2);
        })
    })

    describe('substring for start and end', () => {
        const input = ['abcdefgh']

        bench('rust', () => {
            getChunkRust(input, 2, 5);
        })

        bench('js', () => {
            getChunkJs(input, 2, 5);
        })
    })
})