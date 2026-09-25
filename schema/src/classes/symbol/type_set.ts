import { ITypeSet, IType, SearchOptions, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, SearchEntry } from './base';

export class TypeSet extends Declaration implements ITypeSet, Searchable {
    /**
     * @brief Type signatures
     * @readonly
     */
    readonly types: Record<string, IType>;

    readonly identifier: Identifier = Identifier.TypeSet;

    public constructor(typeSet: ITypeSet) {
        super(typeSet);

        this.types = typeSet.types;
    }

    public searchEntries(options: Readonly<SearchOptions>): SearchEntry[] {
        const ret = super.searchEntries(options);

        const parents = [...options.parents, `${this.identifier}.${this.name}`];

        if (options.l1Only !== true) {
            for (const type of Object.values(this.types)) {
                for (const arg of type.parsedSignature.arguments) {
                    ret.push({
                        name: arg.name,
                        term: arg.type,
                        identifier: Identifier.Argument,
                        part: Part.Parameter,
                        path: [...parents, `${Identifier.Entry}.${type.type}`, `${Identifier.Argument}.${arg.name}`],
                        boost: 0,
                        weight: 0,
                    });
                }

                ret.push({
                    name: type.parsedSignature.returnType,
                    term: type.parsedSignature.returnType,
                    identifier: Identifier.Return,
                    part: Part.Return,
                    path: [...parents, `${Identifier.Return}.${type.parsedSignature.returnType}`],
                    boost: 0,
                    weight: 0,
                });
            }
        }

        if (options.weighted !== false) {
            ret.forEach(e => e.weight += IdentifierWeights.TypeSet);
        }

        return ret;
    }
}
