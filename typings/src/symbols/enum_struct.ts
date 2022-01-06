import { Declaration } from './base';
import { SearchYield } from './search';
import { Function } from './function';
import { Identifier } from './types';

export interface Field extends Declaration {
    tag: Identifier.Field,

    /**
     * @brief Type of the field
     * @readonly
     */
    readonly type: string,
}

export interface EnumStruct extends Declaration {
    tag: Identifier.EnumStruct,

    /**
     * @brief Functions within this enum struct
     * @readonly
     */
    readonly methods: Function[],

    /**
     * @brief Fields within this enum struct
     * @readonly
     */
    readonly fields: Field[],
}

export interface EnumStructYield extends SearchYield {
    readonly tag: Identifier.EnumStruct,

    fields: EnumStruct,
}

export interface EnumStructMethodYield extends SearchYield {
    readonly tag: Identifier.EnumStructMethod,

    fields: [EnumStruct, Function],
}

export interface EnumStructFieldYield extends SearchYield {
    readonly tag: Identifier.EnumStructField,

    fields: [EnumStruct, Field],
}
