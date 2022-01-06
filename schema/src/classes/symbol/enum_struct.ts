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

    public async search(needle: string, options?: SearchOptions): Promise<SearchResult[]> {
        let ret = [
            ...await super.search(needle, options),
        ];

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
                score: calculateScore(field.name, needle),
            });
        }

        if (!options || !options?.weighted) {
            ret = ret.map(e => {
                e.score += IdentifierWeights.EnumStruct;

                return e;
            });
        }

        return ret;
    }
}
