import { symbolDistance } from '@/utils';
import { Declaration, searchDeclaration } from './base';
import { Containment, SymbolYield, SearchYield } from './search';
import { Identifier, IdentifierWeights } from './types';

export interface Entry extends Declaration {
    /**
     * @brief Value that are explicitly set in code expressions
     */
    value?: string,
}

export interface Enumeration extends Declaration {
    tag: Identifier.Enumeration,

    /**
     * @brief Enum entries
     * @readonly
     */
    readonly entries: Entry[],
}

export interface EnumerationYield extends SearchYield {
    readonly tag: Identifier.Enumeration,

    fields: Enumeration,
}
