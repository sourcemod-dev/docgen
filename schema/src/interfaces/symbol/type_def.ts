import { IDeclaration } from './base';
import { IArgument } from './argument';

export interface ITypeDefinition extends IDeclaration {
     /**
     * @brief Function signature
     * @readonly
     */
    readonly type: string;

    /**
     * @brief Parsed function signature
     * @note Null if type is not a function signature, such as `typedef Address = int`
     * @readonly
     */
    readonly parsedSignature: ITypeSignature | null;
}

export interface ITypeSignature {
    /**
     * @brief Return type of the function
     * @readonly
     */
    readonly returnType: string;

    /**
     * @brief Arguments of the function
     * @readonly
     */
    readonly arguments: IArgument[];
}
