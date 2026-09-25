import { IDeclaration, IDocumentation } from './base';
import { ITypeSignature } from './type_def';

/**
 * @note This extends Documentation instead of Declaration
 */
export interface IType extends IDocumentation {
    /**
     * @brief Signature of the function
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

export interface ITypeSet extends IDeclaration {
    /**
     * @brief Type signatures
     * @readonly
     */
    readonly types: Record<string, IType>;
}
