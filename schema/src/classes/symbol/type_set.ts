import { ITypeSet, IType, SearchOptions, SearchResult, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, calculateScore } from './base';

export class TypeSet extends Declaration implements ITypeSet, Searchable {
    /**
     * @brief Type signatures
     * @readonly
     */
     readonly types: IType[];

    public constructor(typeSet: ITypeSet) {
        super(typeSet);

        this.types = typeSet.types;
    }

    public async search(needle: string, options?: SearchOptions): Promise<SearchResult[]> {
        let ret = [
            ...await super.search(needle, options),
        ];

        for (const type of this.types) {
            ret.push({
                name: type.type,
                identifier: Identifier.TypeSet,
                part: Part.Parameter,
                score: calculateScore(type.type, needle),
            });
        }

        if (!options || !options?.weighted) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.TypeSet;
    
                return e;
            });
        }

        return ret;
    }
}
