import { Declaration } from './base';
import { Argument } from './argument';

export interface TypeDefinition extends Declaration {
     /**
     * @brief Function signature
     * @readonly
     */
    readonly type: string,

    /**
     * @brief Parsed function signature
     * @readonly
     */
    readonly parsedSignature: TypeSignature,
}

export interface TypeSignature {
    /**
     * @brief Return type of the function
     * @readonly
     */
    readonly returnType: string,

    /**
     * @brief Arguments of the function
     * @readonly
     */
    readonly arguments: Argument[],
}
