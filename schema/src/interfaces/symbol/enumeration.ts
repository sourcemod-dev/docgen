import { IDeclaration } from './base';

export interface IEntry extends IDeclaration {
    /**
     * @brief Value that are explicitly set in code expressions
     */
    value?: string;
}

export interface IEnumeration extends IDeclaration {
    /**
     * @brief Enum entries
     * @readonly
     */
    readonly entries: IEntry[];
}
