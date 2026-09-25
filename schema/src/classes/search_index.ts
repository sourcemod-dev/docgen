import { SearchOptions, SearchResult } from '../interfaces';
import { SearchEntry, calculateScore, toSearchResult, trackedEntries, recordRecentAdditions } from './symbol/base';

/**
 * @brief Bundle and strand searches only return results scoring above this
 */
export const MIN_SCORE = 0.5;

// Slack for float rounding when pruning, candidates are rescored exactly
const EPSILON = 1e-9;

/**
 * @brief Inverted index over search entries
 *
 * calculateScore ranks a term by exact match, case-insensitive match,
 * case-insensitive substring, and otherwise by Sørensen-Dice similarity of
 * bigrams. A term can only clear MIN_SCORE by containing the needle or by
 * sharing enough bigrams with it, so rather than scoring every entry:
 *
 * - Entries are grouped by term, as many share one (int, void, Handle, ...),
 *   and each distinct term is scored at most once per search.
 * - Substring candidates are read off the posting list of the needle's rarest
 *   lowercase bigram, since a term containing the needle contains all of them.
 * - Dice candidates are found by summing shared bigrams over the needle's
 *   posting lists, which yields the exact coefficient, and keeping the terms
 *   that could still clear MIN_SCORE with the largest bonus in the index.
 *
 * Candidates are then scored with calculateScore, so results are identical
 * to scoring every entry.
 */
export class SearchIndex {
    private readonly entries: SearchEntry[];

    // Term id of each entry
    private readonly entryTerms: number[] = [];

    private readonly terms: string[] = [];

    private readonly lowerTerms: string[] = [];

    // Entry indices of each term
    private readonly termEntries: number[][] = [];

    // Bigram count of each term with whitespace removed
    private readonly bigramCounts: number[] = [];

    // Case-sensitive bigrams of terms with whitespace removed, as compared by Dice
    private readonly bigrams = new Map<number, { terms: number[], counts: number[] }>();

    // Distinct lowercase bigrams of terms, as compared by substring
    private readonly lowerBigrams = new Map<number, number[]>();

    // Largest boost + weight of any entry
    private readonly maxBonus: number;

    // Entries counting towards recent additions, in processing order
    private readonly tracked: number[];

    // Scratch space for shared bigram counts, zeroed after each search
    private readonly shared: Int32Array;

    public constructor(entries: SearchEntry[]) {
        this.entries = entries;

        const termIds = new Map<string, number>();

        let maxBonus = 0;

        entries.forEach((entry, i) => {
            let id = termIds.get(entry.term);

            if (id === undefined) {
                id = this.addTerm(entry.term);
                termIds.set(entry.term, id);
            }

            this.entryTerms.push(id);
            this.termEntries[id].push(i);
            maxBonus = Math.max(maxBonus, entry.boost + entry.weight);
        });

        this.maxBonus = maxBonus;
        this.tracked = trackedEntries(entries);
        this.shared = new Int32Array(this.terms.length);
    }

    /**
     * @brief Search entries scoring above MIN_SCORE, in entry order
     *
     * @param parents Prepended to the path of each result
     */
    public search(needle: string, parents: readonly string[] = []): SearchResult[] {
        const termScores = this.scoreTerms(needle);

        const matches: number[] = [];

        for (const [id, termScore] of termScores) {
            for (const i of this.termEntries[id]) {
                const entry = this.entries[i];

                // Same expression as toSearchResult
                if (termScore + entry.boost + entry.weight > MIN_SCORE) {
                    matches.push(i);
                }
            }
        }

        matches.sort((a, b) => a - b);

        const termScore = (i: number) => {
            const id = this.entryTerms[i];

            return termScores.get(id) ?? calculateScore(this.terms[id], needle);
        };

        recordRecentAdditions(this.entries, this.tracked, i => toSearchResult(this.entries[i], termScore(i), parents));

        return matches.map(i => toSearchResult(this.entries[i], termScore(i), parents));
    }

    /**
     * @brief calculateScore of every term that may clear MIN_SCORE, by term id
     */
    private scoreTerms(needle: string): Map<number, number> {
        const ret = new Map<number, number>();

        const score = (id: number) => {
            if (!ret.has(id)) {
                ret.set(id, calculateScore(this.terms[id], needle));
            }
        };

        const minTermScore = MIN_SCORE - this.maxBonus - EPSILON;

        const lower = needle.toLowerCase();
        const stripped = needle.replace(/\s+/g, '');

        // Nothing to narrow down with, score everything
        if (minTermScore <= 0 || lower.length < 2 || stripped.length < 2) {
            this.terms.forEach((_, id) => score(id));

            return ret;
        }

        // Terms containing the needle
        let rarest: number[] | undefined;

        for (const bigram of countBigrams(lower).keys()) {
            const posting = this.lowerBigrams.get(bigram) ?? [];

            if (rarest === undefined || posting.length < rarest.length) {
                rarest = posting;
            }
        }

        for (const id of rarest!) {
            if (this.lowerTerms[id].includes(lower)) {
                score(id);
            }
        }

        // Terms similar enough to the needle
        const touched: number[] = [];

        for (const [bigram, count] of countBigrams(stripped)) {
            const posting = this.bigrams.get(bigram);

            if (posting === undefined) {
                continue;
            }

            for (let j = 0; j < posting.terms.length; j++) {
                const id = posting.terms[j];

                if (this.shared[id] === 0) {
                    touched.push(id);
                }

                this.shared[id] += Math.min(count, posting.counts[j]);
            }
        }

        const needleBigrams = stripped.length - 1;

        for (const id of touched) {
            if ((2.0 * this.shared[id]) / (this.bigramCounts[id] + needleBigrams) > minTermScore) {
                score(id);
            }

            this.shared[id] = 0;
        }

        return ret;
    }

    private addTerm(term: string): number {
        const id = this.terms.length;

        const lower = term.toLowerCase();
        const stripped = term.replace(/\s+/g, '');

        this.terms.push(term);
        this.lowerTerms.push(lower);
        this.termEntries.push([]);
        this.bigramCounts.push(Math.max(stripped.length - 1, 0));

        for (const [bigram, count] of countBigrams(stripped)) {
            let posting = this.bigrams.get(bigram);

            if (posting === undefined) {
                posting = { terms: [], counts: [] };
                this.bigrams.set(bigram, posting);
            }

            posting.terms.push(id);
            posting.counts.push(count);
        }

        for (const bigram of countBigrams(lower).keys()) {
            let posting = this.lowerBigrams.get(bigram);

            if (posting === undefined) {
                posting = [];
                this.lowerBigrams.set(bigram, posting);
            }

            posting.push(id);
        }

        return id;
    }
}

/**
 * @brief Occurrences of each bigram, keyed by its two UTF-16 code units
 */
function countBigrams(s: string): Map<number, number> {
    const ret = new Map<number, number>();

    for (let i = 0; i < s.length - 1; i++) {
        const bigram = s.charCodeAt(i) * 0x10000 + s.charCodeAt(i + 1);

        ret.set(bigram, (ret.get(bigram) ?? 0) + 1);
    }

    return ret;
}

const indexes = new WeakMap<object, Map<string, SearchIndex>>();

/**
 * @brief Index of owner's entries for the given options, built on first use
 *
 * Indexes are cached per owner and per combination of options that affects
 * entries or scores. Parents are applied per search and don't need their own.
 */
export function getSearchIndex(
    owner: object,
    options: Readonly<SearchOptions>,
    collect: (options: Readonly<SearchOptions>) => SearchEntry[],
): SearchIndex {
    let cache = indexes.get(owner);

    if (cache === undefined) {
        cache = new Map();
        indexes.set(owner, cache);
    }

    const key = JSON.stringify([options.weighted !== false, options.l1Only === true, options.identifier || null]);

    let index = cache.get(key);

    if (index === undefined) {
        index = new SearchIndex(collect({ ...options, parents: [] }));
        cache.set(key, index);
    }

    return index;
}
