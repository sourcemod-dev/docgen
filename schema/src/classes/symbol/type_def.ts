import { ITypeDefinition, ITypeSignature } from '../../interfaces';
import { Declaration } from './base';

export class TypeDefinition extends Declaration implements ITypeDefinition {
    /**
     * @brief Function signature
     * @readonly
     */
     readonly type: string;

     /**
      * @brief Parsed function signature
      * @readonly
      */
     readonly parsedSignature: ITypeSignature;

    public constructor(typeDef: ITypeDefinition) {
        super(typeDef);

        this.type = typeDef.type;
        this.parsedSignature = typeDef.parsedSignature;
    }
}
