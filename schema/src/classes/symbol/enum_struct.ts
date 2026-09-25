import { Declaration, SearchEntry } from './base';
import { Function } from './function';
import { IEnumStruct, IField, Searchable, SearchOptions, Identifier, IdentifierWeights, Part } from '../../interfaces';

export class EnumStruct extends Declaration implements IEnumStruct, Searchable {
    /**
     * @brief Functions within this enum struct
     * @readonly
     */
    readonly methods: Record<string, Function>;

    /**
     * @brief Fields within this enum struct
     * @readonly
     */
    readonly fields: Record<string, Field>;

    readonly identifier: Identifier = Identifier.EnumStruct;

    public constructor(es: IEnumStruct) {
        super(es);

        this.methods = Object.keys(es.methods).reduce((acc, key) => {
            acc[key] = new Function(es.methods[key], Identifier.EnumStructMethod);

            return acc; 
        }, {} as Record<string, Function>);

        this.fields = Object.keys(es.fields).reduce((acc, key) => {
            acc[key] = new Field(es.fields[key]);

            return acc; 
        }, {} as Record<string, Field>);
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
                    identifier: Identifier.EnumStructMethod,
                });

                entries[0].trackOrder = i + 1;

                ret.push(...entries);
            });

            for (const field of Object.values(this.fields)) {
                ret.push({
                    name: field.name,
                    term: field.name,
                    identifier: Identifier.EnumStructField,
                    part: Part.Name,
                    path: [...parents, `${Identifier.EnumStructField}.${field.name}`],
                    boost: 0,
                    weight: 0,
                });
            }
        }

        if (options.weighted !== false) {
            ret.forEach(e => e.weight += IdentifierWeights.EnumStruct);
        }

        return ret;
    }
}

export class Field extends Declaration implements IField {
    /**
     * @brief Type of the field
     * @readonly
     */
    readonly type: string;

    readonly identifier: Identifier = Identifier.Field;

    public constructor(field: IField) {
        super(field);

        this.type = field.type;
    }
}
