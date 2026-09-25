import { Declaration, SearchEntry } from './base';
import { IEnumeration, IEntry, Searchable, Identifier, IdentifierWeights, SearchOptions, Part } from '../../interfaces';

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

    public searchEntries(options: Readonly<SearchOptions>): SearchEntry[] {
        const ret = super.searchEntries(options);

        ret[0].boost += 0.01;

        const parents = [...options.parents, `${this.identifier}.${this.name}`];

        if (options.l1Only !== true) {
            for (const entry of Object.values(this.entries)) {
                ret.push({
                    name: entry.name,
                    term: entry.name,
                    identifier: Identifier.EnumerationEntry,
                    part: Part.Name,
                    path: [...parents, `${Identifier.EnumerationEntry}.${entry.name}`],
                    boost: 0,
                    weight: 0,
                });
            }
        }

        if (options.weighted !== false) {
            ret.forEach(e => e.weight += IdentifierWeights.Enumeration);
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
