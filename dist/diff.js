/**
 * 構造的なS式の差分計算
 * difftasticのアプローチを参考に、AST構造の差分を計算
 */
export class StructuralDiff {
    /**
     * 2つのS式の構造的な差分を計算
     */
    diff(left, right) {
        const operations = [];
        this.diffRecursive(left, right, [], operations);
        return operations;
    }
    diffRecursive(left, right, path, operations) {
        // 完全に同一の場合
        if (this.isEqual(left, right)) {
            operations.push({ type: 'equal', left, right });
            return;
        }
        // 異なる型の場合は置換
        if (left.type !== right.type) {
            operations.push({
                type: 'replace',
                oldValue: left,
                newValue: right,
                path: [...path]
            });
            return;
        }
        // 同じ型の場合の詳細な比較
        switch (left.type) {
            case 'atom':
                if (left.value !== right.value) {
                    operations.push({
                        type: 'replace',
                        oldValue: left,
                        newValue: right,
                        path: [...path]
                    });
                }
                break;
            case 'symbol':
                if (left.name !== right.name) {
                    operations.push({
                        type: 'replace',
                        oldValue: left,
                        newValue: right,
                        path: [...path]
                    });
                }
                break;
            case 'list':
                this.diffLists(left, right, path, operations);
                break;
        }
    }
    diffLists(left, right, path, operations) {
        const leftElements = left.elements;
        const rightElements = right.elements;
        // Myers' diff algorithm の簡易版
        const lcs = this.longestCommonSubsequence(leftElements, rightElements);
        let leftIndex = 0;
        let rightIndex = 0;
        let lcsIndex = 0;
        while (leftIndex < leftElements.length || rightIndex < rightElements.length) {
            if (lcsIndex < lcs.length &&
                leftIndex < leftElements.length &&
                rightIndex < rightElements.length &&
                this.isEqual(leftElements[leftIndex], rightElements[rightIndex])) {
                // 共通要素
                this.diffRecursive(leftElements[leftIndex], rightElements[rightIndex], [...path, leftIndex], operations);
                leftIndex++;
                rightIndex++;
                lcsIndex++;
            }
            else if (lcsIndex < lcs.length &&
                leftIndex < leftElements.length &&
                this.isEqual(leftElements[leftIndex], lcs[lcsIndex])) {
                // 右側に要素が挿入された
                operations.push({
                    type: 'insert',
                    value: rightElements[rightIndex],
                    path: [...path, rightIndex]
                });
                rightIndex++;
            }
            else if (lcsIndex < lcs.length &&
                rightIndex < rightElements.length &&
                this.isEqual(rightElements[rightIndex], lcs[lcsIndex])) {
                // 左側から要素が削除された
                operations.push({
                    type: 'delete',
                    value: leftElements[leftIndex],
                    path: [...path, leftIndex]
                });
                leftIndex++;
            }
            else {
                // 置換または他の複雑な変更
                if (leftIndex < leftElements.length && rightIndex < rightElements.length) {
                    this.diffRecursive(leftElements[leftIndex], rightElements[rightIndex], [...path, leftIndex], operations);
                    leftIndex++;
                    rightIndex++;
                }
                else if (leftIndex < leftElements.length) {
                    operations.push({
                        type: 'delete',
                        value: leftElements[leftIndex],
                        path: [...path, leftIndex]
                    });
                    leftIndex++;
                }
                else if (rightIndex < rightElements.length) {
                    operations.push({
                        type: 'insert',
                        value: rightElements[rightIndex],
                        path: [...path, rightIndex]
                    });
                    rightIndex++;
                }
            }
        }
    }
    longestCommonSubsequence(left, right) {
        const m = left.length;
        const n = right.length;
        const dp = Array(m + 1).fill(null).map(() => Array(n + 1).fill(0));
        for (let i = 1; i <= m; i++) {
            for (let j = 1; j <= n; j++) {
                if (this.isEqual(left[i - 1], right[j - 1])) {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                }
                else {
                    dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
                }
            }
        }
        // バックトラック
        const lcs = [];
        let i = m, j = n;
        while (i > 0 && j > 0) {
            if (this.isEqual(left[i - 1], right[j - 1])) {
                lcs.unshift(left[i - 1]);
                i--;
                j--;
            }
            else if (dp[i - 1][j] > dp[i][j - 1]) {
                i--;
            }
            else {
                j--;
            }
        }
        return lcs;
    }
    isEqual(left, right) {
        if (left.type !== right.type) {
            return false;
        }
        switch (left.type) {
            case 'atom':
                return left.value === right.value;
            case 'symbol':
                return left.name === right.name;
            case 'list':
                const leftList = left;
                const rightList = right;
                if (leftList.elements.length !== rightList.elements.length) {
                    return false;
                }
                for (let i = 0; i < leftList.elements.length; i++) {
                    if (!this.isEqual(leftList.elements[i], rightList.elements[i])) {
                        return false;
                    }
                }
                return true;
        }
    }
}
//# sourceMappingURL=diff.js.map