import { Declaration } from './base';

export interface Entry extends Declaration {
    /**
     * @brief Value that are explicitly set in code expressions
     */
    value?: string,
}

export interface Enumeration extends Declaration {
    /**
     * @brief Enum entries
     * @readonly
     */
    readonly entries: Entry[],
}
