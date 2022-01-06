import { Declaration } from './base';
import { Property } from './property';
import { Function } from './function';

export interface MethodMap extends Declaration {
    /**
     * @brief Parent inheritance if any
     * @readonly
     */
    readonly parent?: string,

    /**
     * @brief Functions within this methodmap
     * @readonly
     */
    readonly methods: Function[],

    /**
     * @brief Properties within this methodmap
     * @readonly
     */
    readonly properties: Property[],
}
