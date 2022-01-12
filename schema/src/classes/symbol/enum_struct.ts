import { Declaration, calculateScore } from './base';
import { Function } from './function';
import { IEnumStruct, IField, Searchable, SearchResult, SearchOptions, Identifier, IdentifierWeights, Part } from '../../interfaces';

export class EnumStruct extends Declaration implements IEnumStruct, Searchable {
    /**
     * @brief Functions within this enum struct
     * @readonly
     */
    readonly methods: Function[];

    /**
     * @brief Fields within this enum struct
     * @readonly
     */
    readonly fields: IField[];

    readonly identifier: Identifier = Identifier.EnumStruct;

    public constructor(es: IEnumStruct) {
        super(es);

        this.methods = es.methods.map(f => new Function(f, Identifier.EnumStructMethod));
        this.fields = es.fields;
    }

    public async search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]> {
        const localOptions = JSON.parse(JSON.stringify(options));

        let ret = [
            ...await super.search(needle, options),
        ];

        ret[0].score += 0.01;

        localOptions.parents.push(`${this.identifier}.${this.name}`);

        if (localOptions.l1Only !== true) {
            for (const method of this.methods) {
                ret.push(...await method.search(needle, {
                    ...options,
                    weighted: false,
                    identifier: Identifier.EnumStructMethod,
                }));
            }

            for (const field of this.fields) {
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
