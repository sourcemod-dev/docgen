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

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        const localOptions = JSON.parse(JSON.stringify(options));

        let ret: SearchResult[] = [
            ...await super.search(needle, localOptions),
        ];

        localOptions.parents.push(`${this.identifier}.${this.name}`);

        if (localOptions.l1Only !== true) {
            for (const type of this.types) {
                for (const arg of type.parsedSignature.arguments) {
                    ret.push({
                        name: arg.name,
                        identifier: Identifier.Argument,
                        part: Part.Parameter,
                        path: [...localOptions.parents, `${Identifier.Entry}.${type.type}`, `${Identifier.Argument}.${arg.name}`],
                        score: calculateScore(arg.type, needle),
                    });
                }

                ret.push({
                    name: type.parsedSignature.returnType,
                    identifier: Identifier.Return,
                    part: Part.Return,
                    path: [...localOptions.parents, `${Identifier.Return}.${type.parsedSignature.returnType}`],
                    score: calculateScore(type.parsedSignature.returnType, needle),
                });
            }
        }

        if (localOptions.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.TypeSet;

                return e;
            });
        }

        return ret;
    }
}
