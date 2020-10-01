import { Declaration, searchDeclaration } from './base';
import { Property } from './property';
import { SearchYield, SymbolYield } from './search';
import { Function } from './function';
import { Identifier, IdentifierWeights } from './types';

export interface MethodMap extends Declaration {
    tag: Identifier.MethodMap,

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

export interface MethodMapYield extends SearchYield {
    readonly tag: Identifier.MethodMap,

    fields: MethodMap,
}

export interface MethodMapMethodYield extends SearchYield {
    readonly tag: Identifier.MethodMapMethod,

    fields: [MethodMap, Function],
}

export interface MethodMapPropertyYield extends SearchYield {
    readonly tag: Identifier.MethodMapProperty,

    fields: [MethodMap, Property],
}
