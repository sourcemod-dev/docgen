import { Declaration, calculateScore } from './base';
import { IEnumeration, IEntry, Searchable, Identifier, IdentifierWeights, SearchOptions, SearchResult, Part } from '../../interfaces';

export class Enumeration extends Declaration implements IEnumeration, Searchable {
    /**
     * @brief Enum entries
     * @readonly
     */
    readonly entries: Record<string, Entry>;

    readonly identifier: Identifier = Identifier.Enumeration;

    public constructor(enumeration: IEnumeration) {
        super(enumeration);

        this.entries = Object.keys(enumeration.entries).reduce((acc, key) => {
            acc[key] = new Entry(enumeration.entries[key]);

            return acc;  
        }, {} as Record<string, Entry>);
    }

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        const localOptions = JSON.parse(JSON.stringify(options));

        let ret = [
            ...await super.search(needle, localOptions),
        ];

        ret[0].score += 0.01;

        localOptions.parents.push(`${this.identifier}.${this.name}`);

        if (localOptions.l1Only !== true) {
            for (const entry of Object.values(this.entries)) {
                ret.push({
                    name: entry.name,
                    identifier: Identifier.EnumerationEntry,
                    part: Part.Name,
                    path: [...localOptions.parents, `${Identifier.EnumerationEntry}.${entry.name}`],
                    score: calculateScore(entry.name, needle),
                });
            }
        }

        if (localOptions.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.Enumeration;

                return e;
            });
        }

        return ret;
    }
}

export class Entry extends Declaration implements IEntry {
    /**
     * @brief Value that are explicitly set in code expressions
     */
    value?: string;

    readonly identifier: Identifier = Identifier.EnumerationEntry;

    constructor(entry: IEntry) {
        super(entry);

        this.value = entry.value;
    }
}
