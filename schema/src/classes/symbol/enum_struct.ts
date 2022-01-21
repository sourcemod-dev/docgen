import { Declaration, calculateScore } from './base';
import { Function } from './function';
import { IEnumStruct, IField, Searchable, SearchResult, SearchOptions, Identifier, IdentifierWeights, Part } from '../../interfaces';

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
                    identifier: Identifier.EnumStructMethod,
                }));
            }

            for (const field of Object.values(this.fields)) {
                ret.push({
                    name: field.name,
                    identifier: Identifier.EnumStructField,
                    part: Part.Name,
                    path: [...localOptions.parents, `${Identifier.EnumStructField}.${field.name}`],
                    score: calculateScore(field.name, needle),
                });
            }
        }

        if (localOptions.weighted !== false) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.EnumStruct;

                return e;
            });
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
