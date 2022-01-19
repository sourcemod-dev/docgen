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

    readonly identifier: Identifier = Identifier.TypeDefinition;

    public constructor(typeDef: ITypeDefinition) {
        super(typeDef);

        this.type = typeDef.type;
        this.parsedSignature = typeDef.parsedSignature;
    }

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        const localOptions = JSON.parse(JSON.stringify(options));

        let ret = [
            ...await super.search(needle, localOptions),
        ];

        localOptions.parents.push(`${this.identifier}.${this.name}`);

        if (localOptions.l1Only !== true) {
            for (const arg of this.parsedSignature.arguments) {
                ret.push({
                    name: arg.type,
                    identifier: Identifier.Argument,
                    part: Part.Parameter,
                    path: [...localOptions.parents, `${Identifier.Argument}.${arg.name}`],
                    score: calculateScore(arg.type, needle),
                });
            }

            ret.push({
                name: this.parsedSignature.returnType,
                identifier: Identifier.Return,
                part: Part.Return,
                path: [...localOptions.parents, `${Identifier.Return}.${this.parsedSignature.returnType}`],
                score: calculateScore(this.parsedSignature.returnType, needle),
            });
        }

        if (localOptions.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.TypeDefinition;

                return e;
            });
        }

        return ret;
    }
}
