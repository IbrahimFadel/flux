#ifndef DEPENDENCY_H
#define DEPENDENCY_H

#include <algorithm>
#include <string>
#include <vector>
#include <filesystem>
#include <memory>
#include "../ast/nodes.h"

namespace fs = std::filesystem;

class DependencyGraph
{
private:
    std::vector<fs::path> paths;
    std::vector<std::pair<int, int>> connections;

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
};

std::unique_ptr<DependencyGraph> createDependencyGraph(const ssc::Nodes &nodes);

#endif