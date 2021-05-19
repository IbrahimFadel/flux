#ifndef DEPENDENCY_H
#define DEPENDENCY_H

#include <algorithm>
#include <string>
#include <vector>
#include <filesystem>
#include <memory>
#include <map>
#include "../ast/nodes.h"

namespace fs = std::filesystem;

namespace ssc
{
    class DependencyGraph
    {
    private:
        std::vector<fs::path> paths;
        std::vector<std::pair<int, int>> connections;

        std::map<fs::path, std::vector<ASTFunctionDefinition *>> functionsToDeclare;

    public:
        void insertConnection(std::pair<int, int> conn)
        {
            if (std::find(connections.begin(), connections.end(), conn) == connections.end())
            {
                connections.push_back(conn);
            }
        };

        int insertPath(fs::path path)
        {
            if (std::find(paths.begin(), paths.end(), path) == paths.end())
            {
                paths.push_back(path);
                return paths.size() - 1;
            }
            return getPathIndex(path);
        };

        int getPathIndex(fs::path path)
        {
            auto it = find(paths.begin(), paths.end(), path);
            if (it != paths.end())
                return it - paths.begin();
            return -1;
        }

        fs::path getPath(int index) { return paths[index]; }

        std::vector<std::pair<int, int>> getConnections()
        {
            return connections;
        }

        void addFunctionsToDeclare(fs::path path, std::vector<ASTFunctionDefinition *> functions) { functionsToDeclare[path] = functions; }
        std::vector<ASTFunctionDefinition *> getFunctionsToDeclare(fs::path path) { return functionsToDeclare[path]; }
    };

    std::unique_ptr<DependencyGraph> createDependencyGraph(const Nodes &nodes);

} // namespace ssc

#endif