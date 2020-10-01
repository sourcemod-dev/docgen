import { symbolDistance } from '@/utils';
import { Declaration, Documentation, searchDeclaration } from './base';
import { SearchYield, SymbolYield, Containment } from './search';
import { Identifier, IdentifierWeights } from './types';
import { TypeSignature } from './type_def';

/**
 * @note This extends Documentation instead of Declaration
 */
export interface Type extends Documentation {
    /**
     * @brief Signature of the function
     * @readonly
     */
    readonly type: string,

    /**
     * @brief Parsed function signature
     * @readonly
     */
    readonly parsedSignature: TypeSignature,
}

export interface TypeSet extends Declaration {
    tag: Identifier.TypeSet,

    /**
     * @brief Type signatures
     * @readonly
     */
    readonly types: Type[],
}

export interface TypeSetYield extends SearchYield {
    readonly tag: Identifier.TypeSet,

    fields: TypeSet,
}
