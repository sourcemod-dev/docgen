import { IFunction, FunctionKind, IArgument, Identifier, Part, Searchable, SearchOptions, IdentifierWeights } from '../../interfaces';
import { Declaration, SearchEntry } from './base';

export class Function extends Declaration implements IFunction, Searchable {
    /**
     * @brief Function kind (forward, stock, etc)
     * @readonly
     */
    readonly kind: FunctionKind;

    /**
     * @brief Return type of the function
     * @readonly
     */
    readonly returnType: string;

    /**
     * @brief Arguments of the function
     * @readonly
     */
    readonly arguments: IArgument[];

    readonly identifier: Identifier = Identifier.Function;

    public constructor(fn: IFunction, identifier?: Identifier) {
        super(fn);

        this.kind = fn.kind;
        this.returnType = fn.returnType;
        this.arguments = fn.arguments;

        if (identifier) {
            this.identifier = identifier;
        }
    }

    public searchEntries(options: Readonly<SearchOptions>): SearchEntry[] {
        const identifier: Identifier = options.identifier ? options.identifier : this.identifier;

        const ret = super.searchEntries(options);

        const parents = [...options.parents, `${identifier}.${this.name}`];

        for (const arg of this.arguments) {
            ret.push({
                name: arg.type,
                term: arg.type,
                identifier,
                part: Part.Parameter,
                path: [...parents, `${Identifier.Argument}.${arg.name}`],
                boost: 0,
                weight: 0,
            });
        }

        ret.push({
            name: this.returnType,
            term: this.returnType,
            identifier,
            part: Part.Return,
            path: [...parents, `${Identifier.Return}.${this.returnType}`],
            boost: 0,
            weight: 0,
        });

        if (options.weighted !== false) {
            ret.forEach(e => e.weight += IdentifierWeights.Function);
        }

        return ret;
    }
}
