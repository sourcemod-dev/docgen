import { IDeclaration } from './base';
import { IFunction } from './function';

export interface IField extends IDeclaration {
    /**
     * @brief Type of the field
     * @readonly
     */
    readonly type: string;
}

export interface IEnumStruct extends IDeclaration {
    /**
     * @brief Functions within this enum struct
     * @readonly
     */
    readonly methods: Record<string, IFunction>;

    /**
     * @brief Fields within this enum struct
     * @readonly
     */
    readonly fields: Record<string, IField>;
}
