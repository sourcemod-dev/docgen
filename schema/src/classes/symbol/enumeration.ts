import { Declaration, calculateScore } from './base';
import { IEnumeration, IEntry, Searchable, Identifier, IdentifierWeights, SearchOptions, SearchResult, Part } from '../../interfaces';

export class Enumeration extends Declaration implements IEnumeration, Searchable {
    /**
     * @brief Enum entries
     * @readonly
     */
    readonly entries: IEntry[];

    readonly identifier: Identifier = Identifier.Enumeration;

    public constructor(enumeration: IEnumeration) {
        super(enumeration);

        this.entries = enumeration.entries;
    }

    public async search(needle: string, options: SearchOptions): Promise<SearchResult[]> {
        let ret = [
            ...await super.search(needle, options),
        ];

        ret[0].score += 0.01;
        
        options.parents.push(this.name);

        for (const entry of this.entries) {
            ret.push({
                name: entry.name,
                identifier: Identifier.Enumeration,
                part: Part.Name,
                path: [...options.parents, this.name, entry.name],
                score: calculateScore(entry.name, needle),
            });
        }

        if (options.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.Enumeration;

                return e;
            });
        }

        return ret;
    }
}
