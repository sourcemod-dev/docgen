import { ITypeDefinition, ITypeSignature, SearchOptions, SearchResult, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, calculateScore } from './base';

export class TypeDefinition extends Declaration implements ITypeDefinition, Searchable {
    /**
     * @brief Function signature
     * @readonly
     */
     readonly type: string;

     /**
      * @brief Parsed function signature
      * @readonly
      */
     readonly parsedSignature: ITypeSignature;

    public constructor(typeDef: ITypeDefinition) {
        super(typeDef);

        this.type = typeDef.type;
        this.parsedSignature = typeDef.parsedSignature;
    }

    public async search(needle: string, options?: SearchOptions): Promise<SearchResult[]> {
        let ret = [
            ...await super.search(needle, options),
        ];

        ret.push({
            name: this.type,
            identifier: Identifier.TypeDefinition,
            part: Part.Parameter,
            score: calculateScore(this.type, needle),
        });

        if (!options || !options?.weighted) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.TypeDefinition;
    
                return e;
            });
        }

        return ret;
    }
}
