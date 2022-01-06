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

    public async search(needle: string, options?: SearchOptions): Promise<SearchResult[]> {
        const identifier: Identifier = (options && options.identifier) ? options.identifier : this.identifier;

        let ret: SearchResult[] = [
            ...await super.search(needle, options),
        ];

        for (const arg of this.arguments) {
            ret.push({
                name: arg.type,
                identifier,
                part: Part.Parameter,
                score: calculateScore(arg.type, needle),
            });
        }

        ret.push({
            name: this.returnType,
            identifier,
            part: Part.Return,
            score: calculateScore(this.returnType, needle),
        });

        if (!options || !options?.weighted) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.Function;

                return e;
            });
        }

        return ret;
    }
}
