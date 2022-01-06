import { Declaration } from './base';

export interface Property extends Declaration {
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
