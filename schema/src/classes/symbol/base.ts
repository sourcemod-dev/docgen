import { IDeclaration, Metadata, Comment, Searchable, SearchResult, SearchOptions, Identifier, Part } from '../../interfaces';

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

        return [{
            name: this.name,
            identifier: this.identifier,
            part: Part.Name,
            path,
            score: calculateScore(this.name, needle),
        }]
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
