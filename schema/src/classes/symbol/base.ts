import { IDeclaration, Comment } from '../../interfaces';

export class Declaration implements IDeclaration {
    /**
     * @brief Declaration name
     * @readonly
     */
    readonly name: string;

    /**
     * @brief Symbol reference line number
     * @readonly
     */
     readonly refLine: number;

     /**
      * @brief Documentation starting byte
      * @readonly
      */
     readonly docStart: number;
 
     /**
      * @brief Documentation ending byte
      * @readonly
      */
     readonly docEnd: number;
 
     /**
      * @brief Parsed documentation
      * @note Null if docStart or docEnd is 0
      * @readonly
      */
     readonly docs: Comment | null;

     public constructor(decl: IDeclaration) {
        this.name = decl.name;
        this.refLine = decl.refLine;
        this.docStart = decl.docStart;
        this.docEnd = decl.docEnd;
        this.docs = decl.docs;
     }
}
