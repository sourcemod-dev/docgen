import { IFunction, FunctionKind, IArgument, Identifier, Part, SearchResult, Searchable, SearchOptions, IdentifierWeights } from '../../interfaces';
import { Declaration, calculateScore } from './base';

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

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        const localOptions = JSON.parse(JSON.stringify(options));

        const identifier: Identifier = localOptions.identifier ? localOptions.identifier : this.identifier;

        let ret: SearchResult[] = [
            ...await super.search(needle, localOptions),
        ];

        localOptions.parents.push(`${identifier}.${this.name}`);

        for (const arg of this.arguments) {
            ret.push({
                name: arg.type,
                identifier,
                part: Part.Parameter,
                path: [...localOptions.parents, `${Identifier.Argument}.${arg.name}`],
                score: calculateScore(arg.type, needle),
            });
        }

        ret.push({
            name: this.returnType,
            identifier,
            part: Part.Return,
            path: [...localOptions.parents, `${Identifier.Return}.${this.returnType}`],
            score: calculateScore(this.returnType, needle),
        });

        if (localOptions.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.Function;

                return e;
            });
        }

        return ret;
    }
}
