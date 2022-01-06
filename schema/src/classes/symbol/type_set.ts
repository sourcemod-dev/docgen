import { ITypeSet, IType, SearchOptions, SearchResult, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, calculateScore } from './base';

export class TypeSet extends Declaration implements ITypeSet, Searchable {
    /**
     * @brief Type signatures
     * @readonly
     */
    readonly types: IType[];

    readonly identifier: Identifier = Identifier.TypeSet;

    public constructor(typeSet: ITypeSet) {
        super(typeSet);

        this.types = typeSet.types;
    }

    public async search(needle: string, options: SearchOptions): Promise<SearchResult[]> {
        let ret: SearchResult[] = [
            ...await super.search(needle, options),
        ];

        options.parents.push(this.name);

        for (const type of this.types) {
            ret.push({
                name: type.type,
                identifier: Identifier.TypeSet,
                part: Part.Parameter,
                path: [...options.parents, type.type],
                score: calculateScore(type.type, needle),
            });
        }

        if (options.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.TypeSet;

                return e;
            });
        }

        return ret;
    }
}
