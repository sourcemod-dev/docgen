import { IMethodMap, IProperty, SearchOptions, Searchable, Identifier, IdentifierWeights, Part } from '../../interfaces';
import { Declaration, SearchEntry } from './base';
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
    readonly properties: Record<string, Property>;

    readonly identifier: Identifier = Identifier.MethodMap;

    public constructor(mm: IMethodMap) {
        super(mm);

        this.parent = mm.parent;

        this.methods = Object.keys(mm.methods).reduce((acc, key) => {
            acc[key] = new Function(mm.methods[key], Identifier.MethodMapMethod);

            return acc; 
        }, {} as Record<string, Function>);

        this.properties = Object.keys(mm.properties).reduce((acc, key) => {
            acc[key] = new Property(mm.properties[key]);

            return acc; 
        }, {} as Record<string, Property>);
    }

    public searchEntries(options: Readonly<SearchOptions>): SearchEntry[] {
        const ret = super.searchEntries(options);

        ret[0].boost += 0.01;

        const parents = [...options.parents, `${this.identifier}.${this.name}`];

        if (options.l1Only !== true) {
            Object.values(this.methods).forEach((method, i) => {
                const entries = method.searchEntries({
                    ...options,
                    parents,
                    weighted: false,
                    identifier: Identifier.MethodMapMethod,
                });

                entries[0].trackOrder = i + 1;

                ret.push(...entries);
            });

            for (const property of Object.values(this.properties)) {
                ret.push({
                    name: property.name,
                    term: property.name,
                    identifier: Identifier.MethodMapProperty,
                    path: [...parents, `${Identifier.MethodMapProperty}.${property.name}`],
                    part: Part.Name,
                    boost: 0,
                    weight: 0,
                });
            }
        }

        if (options.weighted !== false) {
            ret.forEach(e => e.weight += IdentifierWeights.MethodMap);
        }

        return ret;
    }
}

export class Property extends Declaration implements IProperty {
    /**
     * @brief Type of the property
     * @readonly
     */
     readonly type: string;

     /**
      * @brief Whether getter exists
      * @readonly
      */
     readonly getter: boolean;
 
     /**
      * @brief Whether setter exists
      * @readonly
      */
     readonly setter: boolean;

    readonly identifier: Identifier = Identifier.Property;

    public constructor(prop: IProperty) {
        super(prop);

        this.type = prop.type;
        this.getter = prop.getter;
        this.setter = prop.setter;
    }
}
