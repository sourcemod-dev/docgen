import { Declaration } from './base';
import { Function } from './function';

export interface Field extends Declaration {
    /**
     * @brief Type of the field
     * @readonly
     */
    readonly type: string,
}

export interface EnumStruct extends Declaration {
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
