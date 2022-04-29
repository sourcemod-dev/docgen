import { IDeclaration, Metadata, Comment, Searchable, SearchResult, SearchOptions, Identifier, Part } from '../../interfaces';

/**
 * When declaration is constructed,
 * we can intercept some data and store top 50 most recent additions.
 */
let RecentAdditions: {
    sr: SearchResult,
    metadata: Metadata,
}[] = [];

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
        const localOptions = JSON.parse(JSON.stringify(options));

        const path = [...localOptions.parents, `${this.identifier}.${this.name}`];

        const ret = {
            name: this.name,
            identifier: this.identifier,
            part: Part.Name,
            path,
            score: calculateScore(this.name, needle),
        };

        if (this.metadata !== null && this.metadata.created !== null) {
            processAddition(this.metadata, ret);
        }

        return [ret];
    }
}

async function processAddition(metadata: Metadata, sr: SearchResult) {
    const t = {
        sr,
        metadata,
    };

    // Keep only the 20 highest created timestamp metadata
    if (RecentAdditions.length < 20) {
        if (RecentAdditions.includes(t))
            return;

        return RecentAdditions.push(t);
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
