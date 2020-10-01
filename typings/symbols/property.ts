import { Declaration } from './base';
import { Identifier } from './types';

export interface Property extends Declaration {
    tag: Identifier.Property,

    /**
     * @brief Type of the property
     * @readonly
     */
    readonly type: string,

    /**
     * @brief Whether getter exists
     * @readonly
     */
    readonly getter: boolean,

    /**
     * @brief Whether setter exists
     * @readonly
     */
    readonly setter: boolean,
}
