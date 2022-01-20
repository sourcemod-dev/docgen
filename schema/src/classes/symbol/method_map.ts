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
    readonly methods: Record<string, Function>;

    /**
     * @brief Properties within this methodmap
     * @readonly
     */
    readonly properties: Record<string, IProperty>;

    readonly identifier: Identifier = Identifier.MethodMap;

    public constructor(mm: IMethodMap) {
        super(mm);

        this.parent = mm.parent;
        this.methods = Object.keys(mm.methods).reduce((acc, key) => {
            acc[key] = new Function(mm.methods[key], Identifier.MethodMapMethod);

            return acc; 
        }, {} as Record<string, Function>);

        this.properties = mm.properties;
    }

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        const localOptions = JSON.parse(JSON.stringify(options));

        let ret = [
            ...await super.search(needle, localOptions),
        ];

        ret[0].score += 0.01;

        localOptions.parents.push(`${this.identifier}.${this.name}`);

        if (localOptions.l1Only !== true) {
            for (const method of Object.values(this.methods)) {
                ret.push(...await method.search(needle, {
                    ...localOptions,
                    weighted: false,
                    identifier: Identifier.MethodMapMethod,
                }));
            }
    
            for (const property of Object.values(this.properties)) {
                ret.push({
                    name: property.name,
                    identifier: Identifier.MethodMapProperty,
                    path: [...localOptions.parents, `${Identifier.MethodMapProperty}.${property.name}`],
                    part: Part.Name,
                    score: calculateScore(property.name, needle),
                });
            }
        }

        if (localOptions.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.MethodMap;

                return e;
            });
        }

        return ret;
    }
}
