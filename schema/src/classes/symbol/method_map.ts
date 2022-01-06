import { IMethodMap, IProperty, SearchOptions, SearchResult, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, calculateScore } from './base';
import { Function } from './function';

export class MethodMap extends Declaration implements IMethodMap, Searchable {
    /**
     * @brief Parent inheritance if any
     * @readonly
     */
    readonly parent?: string;

    /**
     * @brief Functions within this methodmap
     * @readonly
     */
    readonly methods: Function[];

    /**
     * @brief Properties within this methodmap
     * @readonly
     */
    readonly properties: IProperty[];

    readonly identifier: Identifier = Identifier.MethodMap;

    public constructor(mm: IMethodMap) {
        super(mm);

        this.parent = mm.parent;
        this.methods = mm.methods.map(f => new Function(f, Identifier.MethodMapMethod));
        this.properties = mm.properties;
    }

    public async search(needle: string, options: SearchOptions): Promise<SearchResult[]> {
        let ret = [
            ...await super.search(needle, options),
        ];

        ret[0].score += 0.01;

        options.parents.push(this.name);

        for (const method of this.methods) {
            ret.push(...await method.search(needle, {
                ...options,
                weighted: false,
                identifier: Identifier.MethodMapMethod,
            }));
        }

        for (const property of this.properties) {
            ret.push({
                name: property.name,
                identifier: Identifier.MethodMapProperty,
                path: [...options.parents, this.name],
                part: Part.Name,
                score: calculateScore(property.name, needle),
            });
        }

        if (options.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.MethodMap;

                return e;
            });
        }

        return ret;
    }
}
