import { ITypeDefinition, ITypeSignature, SearchOptions, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, SearchEntry } from './base';

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

    public searchEntries(options: Readonly<SearchOptions>): SearchEntry[] {
        const ret = super.searchEntries(options);

        const parents = [...options.parents, `${this.identifier}.${this.name}`];

        if (options.l1Only !== true) {
            for (const arg of this.parsedSignature.arguments) {
                ret.push({
                    name: arg.type,
                    term: arg.type,
                    identifier: Identifier.Argument,
                    part: Part.Parameter,
                    path: [...parents, `${Identifier.Argument}.${arg.name}`],
                    boost: 0,
                    weight: 0,
                });
            }

            ret.push({
                name: this.parsedSignature.returnType,
                term: this.parsedSignature.returnType,
                identifier: Identifier.Return,
                part: Part.Return,
                path: [...parents, `${Identifier.Return}.${this.parsedSignature.returnType}`],
                boost: 0,
                weight: 0,
            });
        }

        if (options.weighted !== false) {
            ret.forEach(e => e.weight += IdentifierWeights.TypeDefinition);
        }

        return ret;
    }
}
