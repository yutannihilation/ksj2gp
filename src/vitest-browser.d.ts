/// <reference types="vitest/browser/context" />

// The upstream matchers augmentation (`declare module 'vitest'`) is fragile
// under the vite-plus alias (`vitest` → `@voidzero-dev/vite-plus-test`),
// so we declare `expect.element` directly here.
import type { Locator } from '@vitest/browser/context';

declare module 'vitest' {
	type PromisifyAssertion<T> = {
		[K in keyof import('vitest').Assertion<T>]: import('vitest').Assertion<T>[K] extends (
			...args: infer A
		) => infer R
			? (...args: A) => Promise<R>
			: import('vitest').Assertion<T>[K];
	};

	interface ExpectStatic {
		element: <T extends HTMLElement | SVGElement | null | Locator>(
			element: T,
			options?: import('vitest').ExpectPollOptions
		) => PromisifyAssertion<Awaited<HTMLElement | SVGElement | null>>;
	}
}
