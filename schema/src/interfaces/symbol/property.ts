import { IDeclaration } from './base';

export interface IProperty extends IDeclaration {
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
