import { IDeclaration } from './base';
import { IProperty } from './property';
import { IFunction } from './function';

export interface IMethodMap extends IDeclaration {
    /**
     * @brief Parent inheritance if any
     * @readonly
     */
    readonly parent?: string;

    /**
     * @brief Functions within this methodmap
     * @readonly
     */
    readonly methods: Record<string, IFunction>;

    /**
     * @brief Properties within this methodmap
     * @readonly
     */
    readonly properties: Record<string, IProperty>;
}
