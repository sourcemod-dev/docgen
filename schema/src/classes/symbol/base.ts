import { IDeclaration, Metadata, Comment, Searchable, SearchResult, SearchOptions, Identifier, Part } from '../../interfaces';

/**
 * When declaration is constructed,
 * we can intercept some data and store top 50 most recent additions.
 */
let RecentAdditions: {
    sr: SearchResult,
    metadata: Metadata,
}[] = [];

// If recent additions are computed already.
let RecentFinalized = false;

export class Declaration implements IDeclaration, Searchable {
    /**
     * @brief Declaration name
     * @readonly
     */
    readonly name: string;

    /**
     * @brief Symbol reference line number
     * @readonly
     */
    readonly refLine: number;

    /**
     * @brief Documentation starting byte
     * @readonly
     */
    readonly docStart: number;

    /**
     * @brief Documentation ending byte
     * @readonly
     */
    readonly docEnd: number;

    /**
     * @brief Parsed documentation
     * @note Null if docStart or docEnd is 0
     * @readonly
     */
    readonly docs: Comment | null;

    readonly metadata: Metadata | null;

    /**
     * @brief Identifier overriden by inherited classes
     */
    readonly identifier: Identifier = Identifier.Unknown;

    public constructor(decl: IDeclaration) {
        this.name = decl.name;
        this.refLine = decl.refLine;
        this.docStart = decl.docStart;
        this.docEnd = decl.docEnd;
        this.docs = decl.docs;
        this.metadata = decl.metadata;
    }

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        return scoreEntries(this.searchEntries(options), needle);
    }

    /**
     * @brief Everything a search can match on this symbol, without scores
     */
    public searchEntries(options: Readonly<SearchOptions>): SearchEntry[] {
        const entry: SearchEntry = {
            name: this.name,
            term: this.name,
            identifier: this.identifier,
            part: Part.Name,
            path: [...options.parents, `${this.identifier}.${this.name}`],
            boost: 0,
            weight: 0,
        };

        if (this.metadata !== null && this.metadata.created !== null) {
            entry.metadata = this.metadata;
            entry.trackOrder = 0;
        }

        return [entry];
    }
}

/**
 * @brief Unscored search candidate
 *
 * Final score is `calculateScore(term, needle) + boost + weight`.
 */
export interface SearchEntry {
    /**
     * @brief Name reported in the search result
     */
    name: string;

    /**
     * @brief String the needle is scored against, usually the same as name
     */
    term: string;

    identifier: Identifier;

    part: Part;

    path: string[];

    /**
     * @brief Bonus applied before the identifier weight
     */
    boost: number;

    /**
     * @brief Identifier weight, 0 if unweighted
     */
    weight: number;

    /**
     * @brief Set on declaration names that count towards recent additions
     */
    metadata?: Metadata;

    /**
     * @brief Order recent additions are processed in, lower first
     */
    trackOrder?: number;
}

/**
 * @param termScore calculateScore of the entry term against the needle
 * @param parents Prepended to the entry path
 */
export function toSearchResult(entry: SearchEntry, termScore: number, parents: readonly string[] = []): SearchResult {
    return {
        name: entry.name,
        identifier: entry.identifier,
        part: entry.part,
        path: [...parents, ...entry.path],
        score: termScore + entry.boost + entry.weight,
    };
}

function scoreEntries(entries: SearchEntry[], needle: string): SearchResult[] {
    const ret = entries.map(e => toSearchResult(e, calculateScore(e.term, needle)));

    recordRecentAdditions(entries, trackedEntries(entries), i => ret[i]);

    return ret;
}

/**
 * @param tracked Entry indices from trackedEntries
 * @param result Search result of the entry at the given index
 */
export function recordRecentAdditions(
    entries: readonly SearchEntry[],
    tracked: readonly number[],
    result: (i: number) => SearchResult,
) {
    if (RecentFinalized)
        return;

    for (const i of tracked) {
        processAddition(entries[i].metadata!, result(i));
    }
}

/**
 * @brief Indices of entries that count towards recent additions, in processing order
 *
 * Declarations are processed first, then nested methods round-robin across
 * their parents. This mirrors the order the previous async search visited them.
 */
export function trackedEntries(entries: readonly SearchEntry[]): number[] {
    const ret: number[] = [];

    entries.forEach((e, i) => {
        if (e.metadata !== undefined) {
            ret.push(i);
        }
    });

    // Array.prototype.sort is stable
    return ret.sort((a, b) => entries[a].trackOrder! - entries[b].trackOrder!);
}

function processAddition(metadata: Metadata, sr: SearchResult) {
    if (RecentFinalized)
        return;

    // Keep only the 20 highest created timestamp metadata
    if (RecentAdditions.length < 20) {
        RecentAdditions.push({
            sr,
            metadata,
        });

        return;
    }
    
    const sorted = RecentAdditions.sort((a, b) => {
        return b.metadata.created!.count - a.metadata.created!.count;
    });

    // If the oldest timestamp is older than the new one, replace it
    if (sorted[sorted.length - 1].metadata.created!.count < metadata.created!.count) {
        sorted [sorted.length - 1] = {
            sr,
            metadata,
        };

        RecentAdditions = sorted;
    }
}

export function getRecentAddtions(): {
    sr: SearchResult,
    metadata: Metadata,
}[] {
    return RecentAdditions;
}

export function setRecentFinalization(finalized: boolean) {
    RecentFinalized = finalized;

    if (!finalized) {
        RecentAdditions = [];
    }
}

export function calculateScore(a: string, b: string): number {
    if (a === b) {
        return 1.0;
    }

    const aLow = a.toLowerCase();
    const bLow = b.toLowerCase();

    if (aLow === bLow) {
        return 0.9;
    }

    if (aLow.includes(bLow)) {
        return 0.8;
    }

    return compareTwoStrings(a, b);
}

function compareTwoStrings(first: string, second: string) {
    first = first.replace(/\s+/g, '')
    second = second.replace(/\s+/g, '')

    if (first === second) return 1; // identical or empty
    if (first.length < 2 || second.length < 2) return 0; // if either is a 0-letter or 1-letter string

    let firstBigrams = new Map();
    for (let i = 0; i < first.length - 1; i++) {
        const bigram = first.substring(i, i + 2);
        const count = firstBigrams.has(bigram)
            ? firstBigrams.get(bigram) + 1
            : 1;

        firstBigrams.set(bigram, count);
    };

    let intersectionSize = 0;
    for (let i = 0; i < second.length - 1; i++) {
        const bigram = second.substring(i, i + 2);
        const count = firstBigrams.has(bigram)
            ? firstBigrams.get(bigram)
            : 0;

        if (count > 0) {
            firstBigrams.set(bigram, count - 1);
            intersectionSize++;
        }
    }

    return (2.0 * intersectionSize) / (first.length + second.length - 2);
}
