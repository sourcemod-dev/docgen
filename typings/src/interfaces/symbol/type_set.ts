import { Declaration, Documentation } from './base';
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
    /**
     * @brief Type signatures
     * @readonly
     */
    readonly types: Type[],
}
